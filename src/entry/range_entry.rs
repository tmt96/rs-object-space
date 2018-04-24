use std::borrow::Borrow;
use std::iter::empty;
use std::iter::IntoIterator;
use std::ops::RangeBounds;
use std::sync::Arc;

use ordered_float::NotNaN;
use serde_json::value::Value;

use entry::helpers::{convert_float_range, get_all_prims_range, get_primitive_range,
                     remove_all_prims_range, remove_primitive_range, remove_value_arc};
use entry::indexer::RangedIndexer;
use entry::{EfficientEntry, TreeSpaceEntry};

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

macro_rules! impl_range_entry {
    ($($ty:ty)*) => {
        $(
            impl RangeEntry<$ty> for TreeSpaceEntry {
                fn get_range<R>(&self, field: &str, condition: R) -> Option<Value>
                where
                    R: RangeBounds<$ty>,
                {
                    self
                        .get_range_helper(field, condition)
                        .and_then(|arc| {
                            let val: &Value = arc.borrow();
                            Some(val.clone())                            
                        })
                }

                fn get_all_range<'a, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = Value> + 'a>
                where
                    R: RangeBounds<$ty>,
                {
                    self.get_all_range_helper(field, condition)
                }

                fn remove_range<R>(&mut self, field: &str, condition: R) -> Option<Value>
                where
                    R: RangeBounds<$ty>,
                {
                    self
                        .remove_range_helper(field, condition)
                        .and_then(|arc| Arc::try_unwrap(arc).ok())
                }

                fn remove_all_range<'a, R>(&'a mut self, field: &str, condition: R) -> Vec<Value>
                where
                    R: RangeBounds<$ty>,
                {
                    self.remove_all_range_helper(field, condition)
                        .into_iter()
                        .filter_map(|arc| Arc::try_unwrap(arc).ok())
                        .collect()
                }
            }
        )*
    };
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
                    indices
                        .into_iter()
                        .filter_map(|i| {
                            let val = self.remove_value_from_index(&i);
                            val.clone().map(|val| self.indexer.remove(i, &val));
                            val
                        })
                        .collect::<Vec<Value>>()
                }
            }
        )*
    };
}

impl_range_entry!{i64 String f64}
impl_efficient_range_entry!{i64 String f64}

trait RangeValueCollection<T> {
    fn get_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeBounds<T>;

    fn get_all_range_helper<'a, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = Value> + 'a>
    where
        R: RangeBounds<T>;

    fn remove_range_helper<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeBounds<T>;

    fn remove_all_range_helper<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeBounds<T>;
}

macro_rules! impl_range_val_collection {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl RangeValueCollection<$ty> for TreeSpaceEntry {

                fn get_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
                where
                    R: RangeBounds<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref map) => get_primitive_range(map, condition),
                        TreeSpaceEntry::Branch(ref field_map) => field_map
                            .get(field)
                            .and_then(|entry| entry.get_range_helper("", condition)),
                        _ => panic!("Not correct type"),
                    }
                }

                fn get_all_range_helper<'a, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = Value> + 'a>
                where
                    R: RangeBounds<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => Box::new(empty()),
                        TreeSpaceEntry::$path(ref map) => {
                            get_all_prims_range(map, condition)
                        }
                        TreeSpaceEntry::Branch(ref field_map) => field_map
                            .get(field)
                            .map_or(
                                Box::new(empty()),
                                |entry| entry.get_all_range_helper("", condition)
                            ),
                        _ => panic!("Not an int type or a struct holding an int"),
                    }
                }

                fn remove_range_helper<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
                where
                    R: RangeBounds<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => None,
                        TreeSpaceEntry::$path(ref mut map) => remove_primitive_range(map, condition),
                        TreeSpaceEntry::Branch(ref mut object_field_map) => {
                            object_field_map
                                .get_mut(field)
                                .and_then(|entry| entry.remove_range_helper(field, condition))
                                .map(|arc| {
                                    remove_value_arc(object_field_map, &arc);
                                    arc
                                })
                        }
                        _ => panic!("Not correct type"),
                    }
                }

                fn remove_all_range_helper<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
                where
                    R: RangeBounds<$ty>,
                {
                    match *self {
                        TreeSpaceEntry::Null => Vec::new(),
                        TreeSpaceEntry::$path(ref mut map) => remove_all_prims_range(map, condition),
                        TreeSpaceEntry::Branch(ref mut field_map) => {
                            let arc_list = field_map
                                .get_mut(field)
                                .map_or(
                                    Vec::new(),
                                    |entry| entry.remove_all_range_helper(field, condition)
                                 );

                            for arc in &arc_list {
                                remove_value_arc(field_map, arc);
                            }
                            arc_list
                        }
                        _ => panic!("Not correct type"),
                    }
                }
            }
        )*
    };
}

impl_range_val_collection!{[IntLeaf, i64] [StringLeaf, String] [FloatLeaf, NotNaN<f64>]}

impl RangeValueCollection<f64> for TreeSpaceEntry {
    fn get_range_helper<R>(&self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeBounds<f64>,
    {
        self.get_range_helper(field, convert_float_range(condition))
    }

    fn get_all_range_helper<'a, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = Value> + 'a>
    where
        R: RangeBounds<f64>,
    {
        self.get_all_range_helper(field, convert_float_range(condition))
    }

    fn remove_range_helper<R>(&mut self, field: &str, condition: R) -> Option<Arc<Value>>
    where
        R: RangeBounds<f64>,
    {
        self.remove_range_helper(field, convert_float_range(condition))
    }

    fn remove_all_range_helper<R>(&mut self, field: &str, condition: R) -> Vec<Arc<Value>>
    where
        R: RangeBounds<f64>,
    {
        self.remove_all_range_helper(field, convert_float_range(condition))
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
//     fn get_range() {
//         let mut int_entry = TreeSpaceEntry::new();
//         assert_eq!(int_entry.get_range::<i64, _>("", 2..4), None);
//         int_entry.add(3);
//         int_entry.add(5);
//         assert_eq!(int_entry.get_range::<i64, _>("", 2..4), Some(3));
//         assert_ne!(int_entry.get_range::<i64, _>("", 2..4), None);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(
//             test_struct_entry.get_range::<TestStruct, _>("count", 2..4),
//             Some(TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             })
//         );
//         assert!(
//             test_struct_entry
//                 .get_range::<TestStruct, _>("count", 2..4)
//                 .is_some()
//         );

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 5,
//                 name: String::from("Duane"),
//             },
//             gpa: 3.5,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });

//         assert_eq!(
//             compound_struct_entry.get_range::<CompoundStruct, _>("person.count", 2..4),
//             Some(CompoundStruct {
//                 person: TestStruct {
//                     count: 3,
//                     name: String::from("Tuan"),
//                 },
//                 gpa: 3.0,
//             })
//         );
//         assert!(
//             compound_struct_entry
//                 .get_range::<CompoundStruct, _>("person.count", 2..4)
//                 .is_some()
//         );
//     }

//     #[test]
//     fn remove_range() {
//         let mut int_entry = TreeSpaceEntry::new();
//         assert_eq!(int_entry.remove_range::<i64, _>("", 2..4), None);
//         int_entry.add(3);
//         int_entry.add(5);
//         assert_eq!(int_entry.remove_range::<i64, _>("", 2..4), Some(3));
//         assert_eq!(int_entry.remove_range::<i64, _>("", 2..4), None);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(
//             test_struct_entry.remove_range::<TestStruct, _>("count", 2..4),
//             Some(TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             })
//         );
//         assert!(
//             test_struct_entry
//                 .remove_range::<TestStruct, _>("count", 2..4)
//                 .is_none()
//         );

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 5,
//                 name: String::from("Duane"),
//             },
//             gpa: 3.5,
//         });

//         assert_eq!(
//             compound_struct_entry.remove_range::<CompoundStruct, _>("person.count", 2..4),
//             Some(CompoundStruct {
//                 person: TestStruct {
//                     count: 3,
//                     name: String::from("Tuan"),
//                 },
//                 gpa: 3.0,
//             })
//         );
//         assert!(
//             compound_struct_entry
//                 .remove_range::<CompoundStruct, _>("person.count", 2..4)
//                 .is_none()
//         );
//     }

//     #[test]
//     fn get_all_range() {
//         let mut int_entry = TreeSpaceEntry::new();
//         int_entry.add(3);
//         int_entry.add(5);
//         assert_eq!(int_entry.get_all_range::<i64, _>("", 2..4).count(), 1);
//         assert_eq!(int_entry.get_all_range::<i64, _>("", 2..4).count(), 1);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Minh"),
//         });

//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(
//             test_struct_entry
//                 .get_all_range::<TestStruct, _>("count", 2..4)
//                 .count(),
//             2
//         );
//         assert_eq!(
//             test_struct_entry
//                 .get_all_range::<TestStruct, _>("count", 2..4)
//                 .count(),
//             2
//         );

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 5,
//                 name: String::from("Duane"),
//             },
//             gpa: 3.5,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Minh"),
//             },
//             gpa: 3.0,
//         });

//         assert_eq!(
//             compound_struct_entry
//                 .get_all_range::<CompoundStruct, _>("person.count", 2..4)
//                 .count(),
//             2
//         );
//         assert_eq!(
//             compound_struct_entry
//                 .get_all_range::<CompoundStruct, _>("person.count", 2..4)
//                 .count(),
//             2
//         );
//     }

//     #[test]
//     fn remove_all_range() {
//         let mut int_entry = TreeSpaceEntry::new();
//         int_entry.add(3);
//         int_entry.add(5);
//         assert_eq!(int_entry.remove_all_range::<i64, _>("", 2..4).len(), 1);
//         assert_eq!(int_entry.remove_all_range::<i64, _>("", 2..4).len(), 0);

//         let mut test_struct_entry = TreeSpaceEntry::new();
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Tuan"),
//         });
//         test_struct_entry.add(TestStruct {
//             count: 3,
//             name: String::from("Minh"),
//         });

//         test_struct_entry.add(TestStruct {
//             count: 5,
//             name: String::from("Duane"),
//         });

//         assert_eq!(
//             test_struct_entry
//                 .remove_all_range::<TestStruct, _>("count", 2..4)
//                 .len(),
//             2
//         );
//         assert_eq!(
//             test_struct_entry
//                 .remove_all_range::<TestStruct, _>("count", 2..4)
//                 .len(),
//             0
//         );
//         assert_eq!(
//             test_struct_entry
//                 .remove_all_range::<TestStruct, _>("count", 4..)
//                 .len(),
//             1
//         );

//         let mut compound_struct_entry = TreeSpaceEntry::new();
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 5,
//                 name: String::from("Duane"),
//             },
//             gpa: 3.5,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Tuan"),
//             },
//             gpa: 3.0,
//         });
//         compound_struct_entry.add(CompoundStruct {
//             person: TestStruct {
//                 count: 3,
//                 name: String::from("Minh"),
//             },
//             gpa: 3.0,
//         });

//         assert_eq!(
//             compound_struct_entry
//                 .remove_all_range::<CompoundStruct, _>("person.count", 2..4)
//                 .len(),
//             2
//         );
//         assert_eq!(
//             compound_struct_entry
//                 .remove_all_range::<CompoundStruct, _>("person.count", 2..4)
//                 .len(),
//             0
//         );
//         assert_eq!(
//             compound_struct_entry
//                 .remove_all_range::<CompoundStruct, _>("person.count", 4..)
//                 .len(),
//             1
//         );
//     }
// }
