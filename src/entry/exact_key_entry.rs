use std::borrow::Borrow;
use std::iter::IntoIterator;
use std::iter::empty;
use std::sync::Arc;

use ordered_float::NotNaN;
use serde_json::value::Value;

use entry::helpers::get_all_prims_key;
use entry::{TreeSpaceEntry, ValueCollection};

pub trait ExactKeyEntry<U> {
    fn get_key(&self, field: &str, key: &U) -> Option<Value>;

    fn get_all_key<'a>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = Value> + 'a>;

    fn remove_key(&mut self, field: &str, key: &U) -> Option<Value>;

    fn remove_all_key<'a>(&'a mut self, field: &str, key: &U) -> Vec<Value>;
}

impl ExactKeyEntry<i64> for TreeSpaceEntry {
    fn get_key(&self, field: &str, key: &i64) -> Option<Value> {
        match self.get_key_helper(field, key) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                Some(val.clone())
            }
            None => None,
        }
    }

    fn get_all_key<'a>(&'a self, field: &str, key: &i64) -> Box<Iterator<Item = Value> + 'a> {
        self.get_all_key_helper(field, key)
    }

    fn remove_key(&mut self, field: &str, key: &i64) -> Option<Value> {
        match self.remove_key_helper(field, key) {
            Some(arc) => Arc::try_unwrap(arc).ok(),
            None => None,
        }
    }

    fn remove_all_key<'a>(&'a mut self, field: &str, key: &i64) -> Vec<Value> {
        self.remove_all_key_helper(field, key)
            .into_iter()
            .filter_map(|arc| Arc::try_unwrap(arc).ok())
            .collect()
    }
}

impl ExactKeyEntry<String> for TreeSpaceEntry {
    fn get_key(&self, field: &str, key: &String) -> Option<Value> {
        match self.get_key_helper(field, key) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                Some(val.clone())
            }
            None => None,
        }
    }

    fn get_all_key<'a>(&'a self, field: &str, key: &String) -> Box<Iterator<Item = Value> + 'a> {
        self.get_all_key_helper(field, key)
    }

    fn remove_key(&mut self, field: &str, key: &String) -> Option<Value> {
        match self.remove_key_helper(field, key) {
            Some(arc) => Arc::try_unwrap(arc).ok(),
            None => None,
        }
    }

    fn remove_all_key<'a>(&'a mut self, field: &str, key: &String) -> Vec<Value> {
        self.remove_all_key_helper(field, key)
            .into_iter()
            .filter_map(|arc| Arc::try_unwrap(arc).ok())
            .collect()
    }
}

impl ExactKeyEntry<bool> for TreeSpaceEntry {
    fn get_key(&self, field: &str, key: &bool) -> Option<Value> {
        match self.get_key_helper(field, key) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                Some(val.clone())
            }
            None => None,
        }
    }

    fn get_all_key<'a>(&'a self, field: &str, key: &bool) -> Box<Iterator<Item = Value> + 'a> {
        self.get_all_key_helper(field, key)
    }

    fn remove_key(&mut self, field: &str, key: &bool) -> Option<Value> {
        match self.remove_key_helper(field, key) {
            Some(arc) => Arc::try_unwrap(arc).ok(),
            None => None,
        }
    }

    fn remove_all_key<'a>(&'a mut self, field: &str, key: &bool) -> Vec<Value> {
        self.remove_all_key_helper(field, key)
            .into_iter()
            .filter_map(|arc| Arc::try_unwrap(arc).ok())
            .collect()
    }
}

impl ExactKeyEntry<NotNaN<f64>> for TreeSpaceEntry {
    fn get_key(&self, field: &str, key: &NotNaN<f64>) -> Option<Value> {
        match self.get_float_key_helper(field, key) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                Some(val.clone())
            }
            None => None,
        }
    }

    fn get_all_key<'a>(
        &'a self,
        field: &str,
        key: &NotNaN<f64>,
    ) -> Box<Iterator<Item = Value> + 'a> {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::FloatLeaf(ref float_map) => get_all_prims_key(float_map, key),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_all_key("", key),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_key(&mut self, field: &str, key: &NotNaN<f64>) -> Option<Value> {
        match self.remove_float_key(field, key) {
            Some(arc) => Arc::try_unwrap(arc).ok(),
            None => None,
        }
    }

    fn remove_all_key<'a>(&'a mut self, field: &str, key: &NotNaN<f64>) -> Vec<Value> {
        self.remove_all_float_key(field, key)
            .into_iter()
            .filter_map(|arc| Arc::try_unwrap(arc).ok())
            .collect()
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
