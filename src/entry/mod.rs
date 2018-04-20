use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::empty;
use std::sync::Arc;

use indexmap::IndexMap;
use ordered_float::NotNaN;
use serde_json::map::Map;
use serde_json::value::Value;
use serde_json::Number;

mod exact_key_entry;
pub mod helpers;
mod range_entry;

use self::helpers::{get_all_prims_from_map, get_primitive_from_map, remove_object,
                    remove_primitive_from_map};
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

enum ValueIndexer {
    FloatLeaf(BTreeMap<NotNaN<f64>, HashSet<u64>>),
    IntLeaf(BTreeMap<i64, HashSet<u64>>),
    BoolLeaf(BTreeMap<bool, HashSet<u64>>),
    StringLeaf(BTreeMap<String, HashSet<u64>>),
    VecLeaf(HashSet<u64>),
    Branch(HashMap<String, ValueIndexer>),
    Null,
}

pub struct EfficientEntry {
    counter: u64,
    value_map: IndexMap<u64, Arc<Value>>,
    indexer: ValueIndexer,
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
        self.get_helper().map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
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
            TreeSpaceEntry::Branch(ref object_field_map) => object_field_map
                .iter()
                .next()
                .map_or(Box::new(empty()), |(_, value)| value.get_all()),
        }
    }

    pub fn remove(&mut self) -> Option<Value> {
        self.remove_helper()
            .and_then(|arc| Arc::try_unwrap(arc).ok())
    }

    pub fn remove_all(&mut self) -> Vec<Value> {
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

    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        // only parse as f64 if it is actually f64
        // (e.g: accept '64.0' but not '64')
        if num.is_f64() {
            self.add_value(num.as_f64().unwrap(), value);
        } else if let Some(i) = num.as_i64() {
            self.add_value(i, value);
        } else {
            panic!("Not a number!");
        }
    }

    fn add_value_by_array(&mut self, _: Vec<Value>, value: Arc<Value>) {
        if let TreeSpaceEntry::Null = *self {
            *self = TreeSpaceEntry::VecLeaf(Vec::new());
        }

        match *self {
            TreeSpaceEntry::VecLeaf(ref mut vec) => vec.push(value),
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    fn add_value_by_object(&mut self, map: Map<String, Value>, value: Arc<Value>) {
        if let TreeSpaceEntry::Null = *self {
            *self = TreeSpaceEntry::Branch(HashMap::new());
        }

        match *self {
            TreeSpaceEntry::Branch(ref mut hashmap) => for (key, val) in map {
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

impl EfficientEntry {
    pub fn new() -> Self {
        EfficientEntry {
            counter: 0,
            value_map: IndexMap::new(),
            indexer: ValueIndexer::Null,
        }
    }

    pub fn add(&mut self, obj: Value) {
        match obj.clone() {
            Value::Number(num) => self.add_value_by_num(num, Arc::new(obj)),
            Value::Bool(boolean) => self.add_value(boolean, Arc::new(obj)),
            Value::String(string) => self.add_value(string, Arc::new(obj)),
            Value::Array(_) => {
                self.add_value_to_list(Arc::new(obj));
                self.update_array_indexer()
            }
            Value::Object(map) => {
                self.add_value_to_list(Arc::new(obj));
                self.update_object_indexer(map)
            }
            _ => (),
        }
    }

    pub fn get(&self) -> Option<Value> {
        self.value_map.values().next().map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }

    pub fn get_all<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
        Box::new(self.value_map.values().map(|item| {
            let val: &Value = item.borrow();
            val.clone()
        }))
    }

    pub fn remove(&mut self) -> Option<Value> {
        self.value_map.pop().map(|(key, value)| {
            let val: &Value = value.borrow();
            remove_arc_reference(&mut self.indexer, key, val);
            val.clone()
        })
    }

    pub fn remove_all(&mut self) -> Vec<Value> {
        let result = self.get_all().collect();
        *self = EfficientEntry::new();
        result
    }

    //////////////////////////////////////
    // PRIVATE METHODS!!
    //////////////////////////////////////
    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        // only parse as f64 if it is actually f64
        // (e.g: accept '64.0' but not '64')
        if num.is_f64() {
            self.add_value(num.as_f64().unwrap(), value);
        } else if let Some(i) = num.as_i64() {
            self.add_value(i, value);
        } else {
            panic!("Not a number!");
        }
    }

    fn update_array_indexer(&mut self) {
        if let ValueIndexer::Null = self.indexer {
            self.indexer = ValueIndexer::VecLeaf(HashSet::new());
        }
        match self.indexer {
            ValueIndexer::VecLeaf(ref mut set) => {
                set.insert(self.counter);
            }
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    // TODO: refactor this method
    fn update_object_indexer(&mut self, map: Map<String, Value>) {
        if let ValueIndexer::Null = self.indexer {
            self.indexer = ValueIndexer::Branch(HashMap::new());
        }

        match self.indexer {
            ValueIndexer::Branch(ref mut hashmap) => {
                for (key, value) in map {
                    match value.clone() {
                        Value::Object(_) => panic!("Incorrect data type! Found object."),
                        Value::Array(_) => {
                            let indexer = hashmap
                                .entry(key)
                                .or_insert(ValueIndexer::VecLeaf(HashSet::new()));

                            match indexer {
                                ValueIndexer::VecLeaf(ref mut set) => {
                                    set.insert(self.counter);
                                }
                                _ => panic!("Incorrect data type! Found vec."),
                            }
                        }
                        Value::Bool(bool_val) => {
                            let indexer = hashmap
                                .entry(key)
                                .or_insert(ValueIndexer::BoolLeaf(BTreeMap::new()));

                            match indexer {
                                ValueIndexer::BoolLeaf(ref mut map) => {
                                    let set = map.entry(bool_val).or_insert(HashSet::new());
                                    set.insert(self.counter);
                                }
                                _ => panic!("Incorrect data type!"),
                            }
                        }
                        Value::String(string) => {
                            let indexer = hashmap
                                .entry(key)
                                .or_insert(ValueIndexer::StringLeaf(BTreeMap::new()));

                            match indexer {
                                ValueIndexer::StringLeaf(ref mut map) => {
                                    let set = map.entry(string).or_insert(HashSet::new());
                                    set.insert(self.counter);
                                }
                                _ => panic!("Incorrect data type!"),
                            }
                        }
                        Value::Number(num) => {
                            if num.is_f64() {
                                let f = num.as_f64().unwrap();
                                let f = NotNaN::new(f).expect("cannot add an NaN value");
                                let indexer = hashmap
                                    .entry(key)
                                    .or_insert(ValueIndexer::FloatLeaf(BTreeMap::new()));

                                match indexer {
                                    ValueIndexer::FloatLeaf(ref mut map) => {
                                        let set = map.entry(f).or_insert(HashSet::new());
                                        set.insert(self.counter);
                                    }
                                    _ => panic!("Incorrect data type!"),
                                }
                            } else if let Some(i) = num.as_i64() {
                                let indexer = hashmap
                                    .entry(key)
                                    .or_insert(ValueIndexer::IntLeaf(BTreeMap::new()));

                                match indexer {
                                    ValueIndexer::IntLeaf(ref mut map) => {
                                        let set = map.entry(i).or_insert(HashSet::new());
                                        set.insert(self.counter);
                                    }
                                    _ => panic!("Incorrect data type!"),
                                }
                            } else {
                                panic!("Not a number!");
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => panic!("Incorrect data type! Found object."),
        }
    }

    fn add_value_to_list(&mut self, arc: Arc<Value>) {
        self.counter += 1;
        self.value_map.entry(self.counter).or_insert(arc);
    }
}

trait ValueCollection<T> {
    fn add_value(&mut self, field_value: T, arc: Arc<Value>);
}

macro_rules! impl_val_collection {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl ValueCollection<$ty> for TreeSpaceEntry {
                fn add_value(&mut self, field_value: $ty, arc: Arc<Value>) {
                    if let TreeSpaceEntry::Null = *self {
                        *self = TreeSpaceEntry::$path(BTreeMap::new());
                    }

                    match *self {
                        TreeSpaceEntry::$path(ref mut map) => {
                            let vec = map.entry(field_value).or_insert(Vec::new());
                            vec.push(arc);
                        }
                        _ => panic!("Incorrect data type!"),
                    }
                }
            }
        )*
    };
}

macro_rules! impl_efficient_val_collection {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl ValueCollection<$ty> for EfficientEntry {
                fn add_value(&mut self, field_value: $ty, arc: Arc<Value>) {
                    if let ValueIndexer::Null = self.indexer {
                        self.indexer = ValueIndexer::$path(BTreeMap::new());
                    }

                    match self.indexer {
                        ValueIndexer::$path(ref mut map) => {
                            self.counter += 1;
                            self.value_map.entry(self.counter).or_insert(arc);
                            let set = map.entry(field_value).or_insert(HashSet::new());
                            set.insert(self.counter);
                        }
                        _ => panic!("Incorrect data type!"),
                    }
                }
            }
        )*
    };
}

impl_val_collection!{[IntLeaf, i64] [StringLeaf, String] [BoolLeaf, bool] [FloatLeaf, NotNaN<f64>] }

impl_efficient_val_collection!{[IntLeaf, i64] [StringLeaf, String] [BoolLeaf, bool] [FloatLeaf, NotNaN<f64>] }

impl ValueCollection<f64> for TreeSpaceEntry {
    fn add_value(&mut self, field_value: f64, arc: Arc<Value>) {
        self.add_value(
            NotNaN::new(field_value).expect("cannot add an NaN value"),
            arc,
        )
    }
}

impl ValueCollection<f64> for EfficientEntry {
    fn add_value(&mut self, field_value: f64, arc: Arc<Value>) {
        self.add_value(
            NotNaN::new(field_value).expect("cannot add an NaN value"),
            arc,
        )
    }
}

fn remove_arc_reference(value_indexer: &mut ValueIndexer, index: u64, value: &Value) {
    match *value_indexer {
        ValueIndexer::BoolLeaf(ref mut map) => match value {
            Value::Bool(bool_val) => {
                map.get_mut(bool_val).map(|set| set.remove(&index));
            }
            _ => panic!("not correct type! Expect bool value"),
        },
        ValueIndexer::StringLeaf(ref mut map) => match value {
            Value::String(string) => {
                map.get_mut(string).map(|set| set.remove(&index));
            }
            _ => panic!("not correct type! Expect string value"),
        },
        ValueIndexer::FloatLeaf(ref mut map) => match value {
            Value::Number(number) => {
                if number.is_f64() {
                    let key =
                        NotNaN::new(number.as_f64().unwrap()).expect("cannot add an NaN value");
                    map.get_mut(&key).map(|set| set.remove(&index));
                }
            }
            _ => panic!("not correct type! Expect float value"),
        },
        ValueIndexer::IntLeaf(ref mut map) => match value {
            Value::Number(number) => {
                if let Some(i) = number.as_i64() {
                    map.get_mut(&i).map(|set| set.remove(&index));
                }
            }
            _ => panic!("not correct type! Expect int value"),
        },
        ValueIndexer::VecLeaf(ref mut set) => {
            set.remove(&index);
        }
        ValueIndexer::Branch(ref mut path_map) => match value {
            Value::Object(obj) => {
                for (path, val) in obj {
                    if let Some(indexer) = path_map.get_mut(path) {
                        remove_arc_reference(indexer, index, val)
                    }
                }
            }
            _ => panic!("not correct type! Expect object value"),
        },
        _ => (),
    }
}

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
