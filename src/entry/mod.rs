use std::iter::empty;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::borrow::Borrow;
use std::iter::IntoIterator;
use std::collections::range::RangeArgument;

use serde_json::value::{from_value, to_value, Value};
use serde_json::map::Map;
use serde_json::Number;
use serde::ser::Serialize;
use serde::de::Deserialize;

mod conditional_entry;
mod helpers;

use self::helpers::{deflatten, flatten, get_all_prims_from_map, get_primitive_conditional,
                    get_primitive_from_map, remove_all_prims_conditional, remove_object,
                    remove_primitive_conditional, remove_primitive_from_map, remove_value_arc};
pub use entry::conditional_entry::ConditionalEntry;
use not_nan::{FloatIsNaN, NotNaN};

pub enum TreeSpaceEntry {
    FloatLeaf(BTreeMap<NotNaN<f64>, Vec<Arc<Value>>>),
    IntLeaf(BTreeMap<i64, Vec<Arc<Value>>>),
    BoolLeaf(BTreeMap<bool, Vec<Arc<Value>>>),
    StringLeaf(BTreeMap<String, Vec<Arc<Value>>>),
    VecLeaf(Vec<Arc<Value>>),
    Branch(HashMap<String, TreeSpaceEntry>),
    Null,
}

impl TreeSpaceEntry {
    pub fn new() -> Self {
        TreeSpaceEntry::Null
    }

    pub fn add<T>(&mut self, obj: T)
    where
        T: Serialize,
    {
        match to_value(obj) {
            Ok(value) => {
                let flattened_val = flatten(value);
                match flattened_val.clone() {
                    Value::Number(num) => self.add_value_by_num(num, Arc::new(flattened_val)),
                    Value::Bool(boolean) => {
                        self.add_value_by_bool(boolean, Arc::new(flattened_val))
                    }
                    Value::String(string) => {
                        self.add_value_by_string(string, Arc::new(flattened_val))
                    }
                    Value::Array(vec) => self.add_value_by_array(vec, Arc::new(flattened_val)),
                    Value::Object(map) => self.add_value_by_object(map, Arc::new(flattened_val)),
                    _ => (),
                }
            }
            Err(e) => panic!("struct not serializable: {:?}", e),
        }
    }

    pub fn get<T>(&self) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        match self.get_helper() {
            Some(arc) => {
                let val: &Value = arc.borrow();
                from_value(deflatten(val.clone())).ok()
            }
            None => None,
        }
    }

    pub fn get_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::BoolLeaf(ref bool_map) => get_all_prims_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_all_prims_from_map(int_map),
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_all_prims_from_map(float_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_all_prims_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => Box::new(vec.iter().filter_map(|item| {
                let val: &Value = item.borrow();
                from_value(deflatten(val.clone())).ok()
            })),
            TreeSpaceEntry::Branch(ref object_field_map) => {
                if let Some((_, value)) = object_field_map.iter().next() {
                    return value.get_all::<T>();
                }
                Box::new(empty())
            }
        }
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        match self.remove_helper() {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn remove_all<'a, T>(&'a mut self) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        let result = self.get_all::<T>().collect();
        match *self {
            TreeSpaceEntry::BoolLeaf(ref mut bool_map) => *bool_map = BTreeMap::new(),
            TreeSpaceEntry::IntLeaf(ref mut int_map) => *int_map = BTreeMap::new(),
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => *float_map = BTreeMap::new(),
            TreeSpaceEntry::StringLeaf(ref mut string_map) => *string_map = BTreeMap::new(),
            TreeSpaceEntry::VecLeaf(ref mut vec) => *vec = Vec::new(),
            TreeSpaceEntry::Branch(ref mut object_field_map) => *object_field_map = HashMap::new(),
            _ => (),
        }
        result
    }

    fn get_helper(&self) -> Option<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref bool_map) => get_primitive_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_primitive_from_map(int_map),
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_primitive_from_map(float_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_primitive_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => vec.get(0).map(|res| res.clone()),
            TreeSpaceEntry::Branch(ref object_field_map) => {
                if let Some((_, value)) = object_field_map.iter().next() {
                    return value.get_helper();
                }
                None
            }
        }
    }

    fn get_int_conditional_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<i64>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::IntLeaf(ref int_map) => get_primitive_conditional(int_map, condition),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_int_conditional_helper("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn get_string_conditional_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<String>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::StringLeaf(ref string_map) => {
                get_primitive_conditional(string_map, condition)
            }
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_string_conditional_helper("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an string type or a struct holding an string"),
        }
    }

    fn get_bool_conditional_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<bool>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref bool_map) => {
                get_primitive_conditional(bool_map, condition)
            }
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_bool_conditional_helper("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an bool type or a struct holding an bool"),
        }
    }

    fn remove_helper(&mut self) -> Option<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref mut bool_map) => remove_primitive_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref mut int_map) => remove_primitive_from_map(int_map),
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => remove_primitive_from_map(float_map),
            TreeSpaceEntry::StringLeaf(ref mut string_map) => remove_primitive_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref mut vec) => vec.pop(),
            TreeSpaceEntry::Branch(ref mut object_field_map) => remove_object(object_field_map),
        }
    }

    fn remove_int_conditional<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<i64>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::IntLeaf(ref mut int_map) => {
                remove_primitive_conditional(int_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut object_field_map) => {
                let arc = match object_field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_int_conditional(field, condition),
                };

                match arc {
                    Some(arc) => {
                        remove_value_arc(object_field_map, &arc);
                        Some(arc)
                    }
                    None => None,
                }
            }
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_string_conditional<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<String>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::StringLeaf(ref mut string_map) => {
                remove_primitive_conditional(string_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut object_field_map) => {
                let arc = match object_field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_string_conditional(field, condition),
                };

                match arc {
                    Some(arc) => {
                        remove_value_arc(object_field_map, &arc);
                        Some(arc)
                    }
                    None => None,
                }
            }
            _ => panic!("Not an string type or a struct holding an string"),
        }
    }

    fn remove_bool_conditional<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<bool>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref mut bool_map) => {
                remove_primitive_conditional(bool_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut object_field_map) => {
                let arc = match object_field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_bool_conditional(field, condition),
                };

                match arc {
                    Some(arc) => {
                        remove_value_arc(object_field_map, &arc);
                        Some(arc)
                    }
                    None => None,
                }
            }
            _ => panic!("Not an int type or a struct holding an bool"),
        }
    }

    fn remove_all_int_conditional<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeArgument<i64>,
    {
        match *self {
            TreeSpaceEntry::Null => Vec::new(),
            TreeSpaceEntry::IntLeaf(ref mut int_map) => {
                remove_all_prims_conditional(int_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut field_map) => {
                let arc_list = match field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_all_int_conditional(field, condition),
                };

                for arc in arc_list.iter() {
                    remove_value_arc(field_map, arc);
                }
                arc_list
            }
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_all_string_conditional<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeArgument<String>,
    {
        match *self {
            TreeSpaceEntry::Null => Vec::new(),
            TreeSpaceEntry::StringLeaf(ref mut string_map) => {
                remove_all_prims_conditional(string_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut field_map) => {
                let arc_list = match field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_all_string_conditional(field, condition),
                };

                for arc in arc_list.iter() {
                    remove_value_arc(field_map, arc);
                }
                arc_list
            }
            _ => panic!("Not an string type or a struct holding an string"),
        }
    }

    fn remove_all_bool_conditional<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeArgument<bool>,
    {
        match *self {
            TreeSpaceEntry::Null => Vec::new(),
            TreeSpaceEntry::BoolLeaf(ref mut bool_map) => {
                remove_all_prims_conditional(bool_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut field_map) => {
                let arc_list = match field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_all_bool_conditional(field, condition),
                };

                for arc in arc_list.iter() {
                    remove_value_arc(field_map, arc);
                }
                arc_list
            }
            _ => panic!("Not an bool type or a struct holding an bool"),
        }
    }

    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        if let Some(i) = num.as_i64() {
            self.add_value_by_int(i, value);
        } else if let Some(f) = num.as_f64() {
            self.add_value_by_float(f, value);
        } else {
            panic!("Not a number!");
        }
    }

    fn add_value_by_int(&mut self, i: i64, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::IntLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::IntLeaf(ref mut map) => {
                let vec = map.entry(i).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found int."),
        }
    }

    fn add_value_by_float(&mut self, f: f64, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::FloatLeaf(BTreeMap::new());
        }

        let key = NotNaN::new(f).expect("NaN is not allowed");

        match *self {
            TreeSpaceEntry::FloatLeaf(ref mut map) => {
                let vec = map.entry(key).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found float."),
        }
    }

    fn add_value_by_string(&mut self, string: String, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::StringLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::StringLeaf(ref mut map) => {
                let vec = map.entry(string).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found String."),
        }
    }

    fn add_value_by_bool(&mut self, boolean: bool, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::BoolLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::BoolLeaf(ref mut map) => {
                let vec = map.entry(boolean).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found bool."),
        }
    }

    fn add_value_by_array(&mut self, _: Vec<Value>, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::VecLeaf(Vec::new());
        }

        match *self {
            TreeSpaceEntry::VecLeaf(ref mut vec) => vec.push(value),
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    fn add_value_by_object(&mut self, map: Map<String, Value>, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::Branch(HashMap::new());
        }

        match *self {
            TreeSpaceEntry::Branch(ref mut hashmap) => for (key, val) in map.into_iter() {
                let sub_entry = hashmap.entry(key).or_insert(TreeSpaceEntry::Null);
                match val.clone() {
                    Value::Number(num) => sub_entry.add_value_by_num(num, value.clone()),
                    Value::Bool(boolean) => sub_entry.add_value_by_bool(boolean, value.clone()),
                    Value::String(string) => sub_entry.add_value_by_string(string, value.clone()),
                    Value::Array(vec) => sub_entry.add_value_by_array(vec, value.clone()),
                    Value::Object(_) => panic!("Incorrect data type! Found object."),
                    _ => (),
                }
            },
            _ => panic!("Incorrect data type! Found object."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct TestStruct {
        count: i32,
        name: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct CompoundStruct {
        person: TestStruct,
        gpa: f64,
    }

    #[test]
    fn get() {
        let mut string_entry = TreeSpaceEntry::new();
        assert_eq!(string_entry.get::<String>(), None);
        string_entry.add(String::from("Hello World"));
        assert_eq!(
            string_entry.get::<String>(),
            Some(String::from("Hello World"))
        );
        assert_ne!(string_entry.get::<String>(), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        assert_eq!(
            test_struct_entry.get::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
            gpa: 3.0,
        });
        assert_eq!(
            compound_struct_entry.get::<CompoundStruct>(),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
                gpa: 3.0,
            })
        );
        assert!(compound_struct_entry.get::<CompoundStruct>().is_some());
    }

    #[test]
    fn remove() {
        let mut entry = TreeSpaceEntry::new();
        assert_eq!(entry.remove::<String>(), None);
        entry.add(String::from("Hello World"));
        assert_eq!(entry.remove::<String>(), Some(String::from("Hello World")));
        assert_eq!(entry.remove::<String>(), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        assert_eq!(
            test_struct_entry.remove::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert_eq!(test_struct_entry.remove::<TestStruct>(), None);

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
            gpa: 3.0,
        });
        assert_eq!(
            compound_struct_entry.remove::<CompoundStruct>(),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
                gpa: 3.0,
            })
        );
        assert!(compound_struct_entry.remove::<CompoundStruct>().is_none());
    }

    #[test]
    fn get_all() {
        let mut entry = TreeSpaceEntry::new();
        assert_eq!(entry.get_all::<String>().count(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        assert_eq!(
            entry.get_all::<String>().collect::<Vec<String>>(),
            vec!["Hello", "World"]
        );
        assert_ne!(entry.get_all::<String>().count(), 0);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(test_struct_entry.get_all::<TestStruct>().count(), 2);
        assert_eq!(test_struct_entry.get_all::<TestStruct>().count(), 2);
    }

    #[test]
    fn remove_all() {
        let mut entry = TreeSpaceEntry::new();
        assert_eq!(entry.remove_all::<String>().len(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        assert_eq!(entry.remove_all::<String>(), vec!["Hello", "World"]);
        assert_eq!(entry.remove_all::<String>().len(), 0);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(test_struct_entry.remove_all::<TestStruct>().len(), 2);
        assert_eq!(test_struct_entry.remove_all::<TestStruct>().len(), 0);
    }
}
