use entry::helpers::get_all_prims_key;
use std::borrow::Borrow;
use std::collections::range::RangeArgument;
use std::collections::{BTreeMap, HashMap};
use std::iter::IntoIterator;
use std::iter::empty;
use std::sync::Arc;

use ordered_float::NotNaN;
use serde_json::Number;
use serde_json::map::Map;
use serde_json::value::Value;

mod exact_key_entry;
pub mod helpers;
mod range_entry;

use self::helpers::{get_all_prims_from_map, get_all_prims_range, get_primitive_from_map,
                    get_primitive_key, get_primitive_range, remove_all_prims_key,
                    remove_all_prims_range, remove_object, remove_primitive_from_map,
                    remove_primitive_key, remove_primitive_range, remove_value_arc};
pub use entry::exact_key_entry::ExactKeyEntry;
pub use entry::range_entry::RangeEntry;

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

    pub fn add(&mut self, obj: Value) {
        match obj.clone() {
            Value::Number(num) => self.add_value_by_num(num, Arc::new(obj)),
            Value::Bool(boolean) => self.add_value(boolean, Arc::new(obj)),
            Value::String(string) => self.add_value(string, Arc::new(obj)),
            Value::Array(vec) => self.add_value_by_array(vec, Arc::new(obj)),
            Value::Object(map) => self.add_value_by_object(map, Arc::new(obj)),
            _ => (),
        }
    }

    pub fn get(&self) -> Option<Value> {
        match self.get_helper() {
            Some(arc) => {
                let val: &Value = arc.borrow();
                Some(val.clone())
            }
            None => None,
        }
    }

    pub fn get_all<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::BoolLeaf(ref bool_map) => get_all_prims_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_all_prims_from_map(int_map),
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_all_prims_from_map(float_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_all_prims_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => Box::new(vec.iter().map(|item| {
                let val: &Value = item.borrow();
                val.clone()
            })),
            TreeSpaceEntry::Branch(ref object_field_map) => {
                if let Some((_, value)) = object_field_map.iter().next() {
                    return value.get_all();
                }
                Box::new(empty())
            }
        }
    }

    pub fn remove(&mut self) -> Option<Value> {
        match self.remove_helper() {
            Some(arc) => Arc::try_unwrap(arc).ok(),
            None => None,
        }
    }

    pub fn remove_all<'a>(&'a mut self) -> Vec<Value> {
        let result = self.get_all().collect();
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

    fn get_float_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<NotNaN<f64>>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_primitive_range(float_map, condition),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_float_range_helper("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an float type or a struct holding an float"),
        }
    }

    fn get_float_key_helper(&self, field: &str, key: &NotNaN<f64>) -> Option<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_primitive_key(float_map, key),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_float_key_helper("", key),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
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

    fn remove_float_range<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<NotNaN<f64>>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => {
                remove_primitive_range(float_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut object_field_map) => {
                let arc = match object_field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_float_range(field, condition),
                };

                match arc {
                    Some(arc) => {
                        remove_value_arc(object_field_map, &arc);
                        Some(arc)
                    }
                    None => None,
                }
            }
            _ => panic!("Not an float type or a struct holding an float"),
        }
    }

    fn remove_float_key(&mut self, field: &str, key: &NotNaN<f64>) -> Option<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => remove_primitive_key(float_map, key),
            TreeSpaceEntry::Branch(ref mut object_field_map) => {
                let arc = match object_field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_float_key(field, key),
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

    fn remove_all_float_range<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeArgument<NotNaN<f64>>,
    {
        match *self {
            TreeSpaceEntry::Null => Vec::new(),
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => {
                remove_all_prims_range(float_map, condition)
            }
            TreeSpaceEntry::Branch(ref mut field_map) => {
                let arc_list = match field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_all_float_range(field, condition),
                };

                for arc in arc_list.iter() {
                    remove_value_arc(field_map, arc);
                }
                arc_list
            }
            _ => panic!("Not an float type or a struct holding an float"),
        }
    }

    fn remove_all_float_key(&mut self, field: &str, key: &NotNaN<f64>) -> Vec<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => Vec::new(),
            TreeSpaceEntry::FloatLeaf(ref mut float_map) => remove_all_prims_key(float_map, key),
            TreeSpaceEntry::Branch(ref mut field_map) => {
                let arc_list = match field_map.get_mut(field) {
                    None => panic!("Field {} does not exist", field),
                    Some(entry) => entry.remove_all_float_key(field, key),
                };

                for arc in arc_list.iter() {
                    remove_value_arc(field_map, arc);
                }
                arc_list
            }
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        if let Some(i) = num.as_i64() {
            self.add_value(i, value);
        } else if let Some(f) = num.as_f64() {
            self.add_value_by_float(f, value);
        } else {
            panic!("Not a number!");
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
                    Value::Bool(boolean) => sub_entry.add_value(boolean, value.clone()),
                    Value::String(string) => sub_entry.add_value(string, value.clone()),
                    Value::Array(vec) => sub_entry.add_value_by_array(vec, value.clone()),
                    Value::Object(_) => panic!("Incorrect data type! Found object."),
                    _ => (),
                }
            },
            _ => panic!("Incorrect data type! Found object."),
        }
    }
}

trait ValueCollection<T> {
    fn add_value(&mut self, criteria: T, value: Arc<Value>);

    fn get_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<T>;

    fn get_all_range_helper<'a, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = Value> + 'a>
    where
        R: RangeArgument<T>;

    fn remove_range_helper<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeArgument<T>;

    fn remove_all_range_helper<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeArgument<T>;

    fn get_key_helper(&self, field: &str, key: &T) -> Option<Arc<Value>>;

    fn get_all_key_helper<'a>(&'a self, field: &str, key: &T) -> Box<Iterator<Item = Value> + 'a>;

    fn remove_key_helper(&mut self, field: &str, key: &T) -> Option<Arc<Value>>;

    fn remove_all_key_helper(&mut self, field: &str, key: &T) -> Vec<Arc<Value>>;
}

macro_rules! impl_val_collection {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl ValueCollection<$ty> for TreeSpaceEntry {
                fn add_value(&mut self, criteria: $ty, value: Arc<Value>) {
                    if let &mut TreeSpaceEntry::Null = self {
                        *self = TreeSpaceEntry::$path(BTreeMap::new());
                    }

                    match *self {
                        TreeSpaceEntry::$path(ref mut map) => {
                            let vec = map.entry(criteria).or_insert(Vec::new());
                            vec.push(value);
                        }
                        _ => panic!("Incorrect data type!"),
                    }
                }

                fn get_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
                where
                    R: RangeArgument<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref map) => get_primitive_range(map, condition),
                        TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                            Some(entry) => entry.get_range_helper("", condition),
                            None => panic!("No such field found!"),
                        },
                        _ => panic!("Not correct type"),
                    }
                }

                fn get_all_range_helper<'a, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = Value> + 'a>
                where
                    R: RangeArgument<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => Box::new(empty()),
                        TreeSpaceEntry::$path(ref map) => {
                            get_all_prims_range(map, condition)
                        }
                        TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                            Some(entry) => entry.get_all_range_helper("", condition),
                            None => panic!("No such field found!"),
                        },
                        _ => panic!("Not an int type or a struct holding an int"),
                    }
                }

                fn remove_range_helper<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
                where
                    R: RangeArgument<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref mut map) => remove_primitive_range(map, condition),
                        TreeSpaceEntry::Branch(ref mut object_field_map) => {
                            let arc = match object_field_map.get_mut(field) {
                                None => panic!("Field {} does not exist", field),
                                Some(entry) => entry.remove_range_helper(field, condition),
                            };

                            match arc {
                                Some(arc) => {
                                    remove_value_arc(object_field_map, &arc);
                                    Some(arc)
                                }
                                None => None,
                            }
                        }
                        _ => panic!("Not correct type"),
                    }
                }

                fn remove_all_range_helper<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
                where
                    R: RangeArgument<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => Vec::new(),
                        TreeSpaceEntry::$path(ref mut map) => remove_all_prims_range(map, condition),
                        TreeSpaceEntry::Branch(ref mut field_map) => {
                            let arc_list = match field_map.get_mut(field) {
                                None => panic!("Field {} does not exist", field),
                                Some(entry) => entry.remove_all_range_helper(field, condition),
                            };

                            for arc in arc_list.iter() {
                                remove_value_arc(field_map, arc);
                            }
                            arc_list
                        }
                        _ => panic!("Not correct type"),
                    }
                }


                fn get_key_helper(&self, field: &str, key: &$ty) -> Option<Arc<Value>> {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref map) => get_primitive_key(map, key),
                        TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                            Some(entry) => entry.get_key_helper("", key),
                            None => panic!("No such field found!"),
                        },
                        _ => panic!("Not correct type"),
                    }
                }

                fn get_all_key_helper<'a>(&'a self, field: &str, key: &$ty) -> Box<Iterator<Item = Value> + 'a> {
                    match *self {
                        TreeSpaceEntry::Null => Box::new(empty()),
                        TreeSpaceEntry::$path(ref int_map) => get_all_prims_key(int_map, &key),
                        TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                            Some(entry) => entry.get_all_key_helper("", key),
                            None => panic!("No such field found!"),
                        },
                        _ => panic!("Not an int type or a struct holding an int"),
                    }
                }


                fn remove_key_helper(&mut self, field: &str, key: &$ty) -> Option<Arc<Value>> {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref mut map) => remove_primitive_key(map, key),
                        TreeSpaceEntry::Branch(ref mut object_field_map) => {
                            let arc = match object_field_map.get_mut(field) {
                                None => panic!("Field {} does not exist", field),
                                Some(entry) => entry.remove_key_helper(field, key),
                            };

                            match arc {
                                Some(arc) => {
                                    remove_value_arc(object_field_map, &arc);
                                    Some(arc)
                                }
                                None => None,
                            }
                        }
                        _ => panic!("Not correct type"),
                    }
                }
                fn remove_all_key_helper(&mut self, field: &str, key: &$ty) -> Vec<Arc<Value>> {
                    match *self {
                        TreeSpaceEntry::Null => Vec::new(),
                        TreeSpaceEntry::$path(ref mut map) => remove_all_prims_key(map, key),
                        TreeSpaceEntry::Branch(ref mut field_map) => {
                            let arc_list = match field_map.get_mut(field) {
                                None => panic!("Field {} does not exist", field),
                                Some(entry) => entry.remove_all_key_helper(field, key),
                            };

                            for arc in arc_list.iter() {
                                remove_value_arc(field_map, arc);
                            }
                            arc_list
                        }
                        _ => panic!("Not an correct type"),
                    }
                }
            }
        )*
    };
}

impl_val_collection!{[IntLeaf, i64] [StringLeaf, String] [BoolLeaf, bool] }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
//     struct TestStruct {
//         count: i32,
//         name: String,
//     }

//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct CompoundStruct {
//         person: TestStruct,
//         gpa: f64,
//     }

//     #[test]
//     fn get() {
//         let mut string_entry = TreeSpaceEntry::new();
//         assert_eq!(string_entry.get::<String>(), None);
//         string_entry.add(String::from("Hello World"));
//         assert_eq!(
//             string_entry.get::<String>(),
//             Some(String::from("Hello World"))
//         );
//         assert_ne!(string_entry.get::<String>(), None);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         assert_eq!(
//             test_struct_entry.get::<TestStruct>(),
//             Some(TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             })
//         );

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });
//         assert_eq!(
//             compound_struct_entry.get::<CompoundStruct>(),
//             Some(CompoundStruct {
//                 person: TestStruct {
//                     count: 3,
//                     name: String::from("Tuan"),
//                 },
//                 gpa: 3.0,
//             })
//         );
//         assert!(compound_struct_entry.get::<CompoundStruct>().is_some());
//     }

//     #[test]
//     fn remove() {
//         let mut entry = TreeSpaceEntry::new();
//         assert_eq!(entry.remove::<String>(), None);
//         entry.add(String::from("Hello World"));
//         assert_eq!(entry.remove::<String>(), Some(String::from("Hello World")));
//         assert_eq!(entry.remove::<String>(), None);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         assert_eq!(
//             test_struct_entry.remove::<TestStruct>(),
//             Some(TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             })
//         );
//         assert_eq!(test_struct_entry.remove::<TestStruct>(), None);

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });
//         assert_eq!(
//             compound_struct_entry.remove::<CompoundStruct>(),
//             Some(CompoundStruct {
//                 person: TestStruct {
//                     count: 3,
//                     name: String::from("Tuan"),
//                 },
//                 gpa: 3.0,
//             })
//         );
//         assert!(compound_struct_entry.remove::<CompoundStruct>().is_none());
//     }

//     #[test]
//     fn get_all() {
//         let mut entry = TreeSpaceEntry::new();
//         assert_eq!(entry.get_all::<String>().count(), 0);
//         entry.add("Hello".to_string());
//         entry.add("World".to_string());
//         assert_eq!(
//             entry.get_all::<String>().collect::<Vec<String>>(),
//             vec!["Hello", "World"]
//         );
//         assert_ne!(entry.get_all::<String>().count(), 0);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(test_struct_entry.get_all::<TestStruct>().count(), 2);
//         assert_eq!(test_struct_entry.get_all::<TestStruct>().count(), 2);
//     }

//     #[test]
//     fn remove_all() {
//         let mut entry = TreeSpaceEntry::new();
//         assert_eq!(entry.remove_all::<String>().len(), 0);
//         entry.add("Hello".to_string());
//         entry.add("World".to_string());
//         assert_eq!(entry.remove_all::<String>(), vec!["Hello", "World"]);
//         assert_eq!(entry.remove_all::<String>().len(), 0);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(test_struct_entry.remove_all::<TestStruct>().len(), 2);
//         assert_eq!(test_struct_entry.remove_all::<TestStruct>().len(), 0);
//     }
// }
