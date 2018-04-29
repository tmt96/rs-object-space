use std::borrow::Borrow;
use std::ops::RangeBounds;
use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::value::Value;

pub mod indexer;

use entry::indexer::{KeyedIndexer, RangedIndexer, ValueIndexer};

pub struct EfficientEntry {
    counter: u64,
    value_map: IndexMap<u64, Arc<Value>>,
    indexer: ValueIndexer,
}

impl EfficientEntry {
    pub fn new() -> Self {
        EfficientEntry {
            counter: 0,
            value_map: IndexMap::new(),
            indexer: ValueIndexer::new(),
        }
    }

    pub fn add(&mut self, obj: Value) {
        self.add_value_to_list(Arc::new(obj.clone()));
        self.indexer.add(obj, self.counter)
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
            self.indexer.remove(key, val);
            val.clone()
        })
    }

    pub fn remove_all(&mut self) -> Vec<Value> {
        let result = self.get_all().collect();
        *self = EfficientEntry::new();
        result
    }

    fn add_value_to_list(&mut self, arc: Arc<Value>) {
        self.counter += 1;
        self.value_map.entry(self.counter).or_insert(arc);
    }

    fn get_value_from_index(&self, index: &u64) -> Option<Value> {
        self.value_map.get(index).map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }

    fn remove_value_from_index(&mut self, index: &u64) -> Option<Value> {
        self.value_map.remove(index).map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }
}

pub trait ExactKeyEntry<U> {
    fn get_key(&self, field: &str, key: &U) -> Option<Value>;

    fn get_all_key<'a>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = Value> + 'a>;

    fn remove_key(&mut self, field: &str, key: &U) -> Option<Value>;

    fn remove_all_key(&mut self, field: &str, key: &U) -> Vec<Value>;
}

macro_rules! impl_efficient_key_entry {
    ($($ty:ty)*) => {
        $(            
            impl ExactKeyEntry<$ty> for EfficientEntry {
                fn get_key(&self, field: &str, key: &$ty) -> Option<Value> {
                    let index = self.indexer.get_index_by_key(field, key);
                    index.and_then(|i| self.get_value_from_index(&i))
                }

                fn get_all_key<'a>(&'a self, field: &str, key: &$ty) -> Box<Iterator<Item = Value> + 'a> {
                    let indices = self.indexer.get_all_indices_by_key(field, key);
                    Box::new(
                        indices.filter_map(move |i| self.get_value_from_index(&i))
                    )
                }

                fn remove_key(&mut self, field: &str, key: &$ty) -> Option<Value> {
                    let index = self.indexer.get_index_by_key(field, key);
                    index.and_then(|i| {
                        let val = self.remove_value_from_index(&i);
                        val.clone().map(|val| self.indexer.remove(i, &val));
                        val
                    })
                }

                fn remove_all_key(&mut self, field: &str, key: &$ty) -> Vec<Value> {
                    let indices: Vec<u64> = self.indexer.get_all_indices_by_key(field, key).collect();
                    let mut result = Vec::new();
                    for i in indices {
                        if let Some(val) = self.remove_value_from_index(&i) {
                            self.indexer.remove(i, &val.clone());
                            result.push(val);
                        }
                    }
                    result
                }
            }
        )*
    };
}

impl_efficient_key_entry!{i64 String bool f64}

pub trait RangeEntry<U> {
    fn get_range<R>(&self, field: &str, condition: R) -> Option<Value>
    where
        R: RangeBounds<U>;

    fn get_all_range<'a, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = Value> + 'a>
    where
        R: RangeBounds<U>;

    fn remove_range<R>(&mut self, field: &str, condition: R) -> Option<Value>
    where
        R: RangeBounds<U>;

    fn remove_all_range<'a, R>(&'a mut self, field: &str, condition: R) -> Vec<Value>
    where
        R: RangeBounds<U>;
}

macro_rules! impl_efficient_range_entry {
    ($($ty:ty)*) => {
        $(            
            impl RangeEntry<$ty> for EfficientEntry {
                fn get_range<R>(&self, field: &str, range: R) -> Option<Value> 
                where R: RangeBounds<$ty>
                {
                    let index = self.indexer.get_index_by_range(field, range);
                    index.and_then(|i| self.get_value_from_index(&i))
                }

                fn get_all_range<'a, R>(&'a self, field: &str, range: R) -> Box<Iterator<Item = Value> + 'a> 
                where R: RangeBounds<$ty>
                {
                    let indices = self.indexer.get_all_indices_by_range(field, range);
                    Box::new(
                        indices.filter_map(move |i| self.get_value_from_index(&i))
                    )
                }

                fn remove_range<R>(&mut self, field: &str, range: R) -> Option<Value> 
                where R: RangeBounds<$ty>
                {
                    let index = self.indexer.get_index_by_range(field, range);
                    index.and_then(|i| {
                        let val = self.remove_value_from_index(&i);
                        val.clone().map(|val| self.indexer.remove(i, &val));
                        val
                    })
                }

                fn remove_all_range<R>(&mut self, field: &str, range: R) -> Vec<Value> 
                where R: RangeBounds<$ty>
                {
                    let indices: Vec<u64> = self.indexer.get_all_indices_by_range(field, range).collect();
                    let mut result = Vec::new();
                    for i in indices {
                        if let Some(val) = self.remove_value_from_index(&i) {
                            self.indexer.remove(i, &val.clone());
                            result.push(val);
                        }
                    }
                    result
                }
            }
        )*
    };
}

impl_efficient_range_entry!{i64 String f64}

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
