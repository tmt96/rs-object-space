use std::iter::empty;
use std::sync::Arc;
use std::borrow::Borrow;
use std::iter::IntoIterator;
use std::collections::range::RangeArgument;

use serde_json::value::{from_value, Value};
use serde::ser::Serialize;
use serde::de::Deserialize;

use entry::helpers::{deflatten, get_all_prims_conditional};
use entry::TreeSpaceEntry;

pub trait ConditionalEntry<U> {
    fn get_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<U>;

    fn get_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U>;

    fn remove_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<U>;

    fn remove_all_conditional<'a, T, R>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U>;
}

impl ConditionalEntry<i64> for TreeSpaceEntry {
    fn get_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<i64>,
    {
        match self.get_int_conditional_helper(field, condition) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                from_value(deflatten(val.clone())).ok()
            }
            None => None,
        }
    }

    fn get_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64>,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_all_prims_conditional(int_map, condition),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_all_conditional("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<i64>,
    {
        match self.remove_int_conditional(field, condition) {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn remove_all_conditional<'a, T, R>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64>,
    {
        self.remove_all_int_conditional(field, condition)
            .into_iter()
            .filter_map(|arc| match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            })
            .collect()
    }
}

impl ConditionalEntry<String> for TreeSpaceEntry {
    fn get_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<String>,
    {
        match self.get_string_conditional_helper(field, condition) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                from_value(deflatten(val.clone())).ok()
            }
            None => None,
        }
    }

    fn get_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String>,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::StringLeaf(ref string_map) => {
                get_all_prims_conditional(string_map, condition)
            }
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_all_conditional("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<String>,
    {
        match self.remove_string_conditional(field, condition) {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn remove_all_conditional<'a, T, R>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String>,
    {
        self.remove_all_string_conditional(field, condition)
            .into_iter()
            .filter_map(|arc| match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            })
            .collect()
    }
}

impl ConditionalEntry<bool> for TreeSpaceEntry {
    fn get_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<bool>,
    {
        match self.get_bool_conditional_helper(field, condition) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                from_value(deflatten(val.clone())).ok()
            }
            None => None,
        }
    }

    fn get_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool>,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::BoolLeaf(ref bool_map) => {
                get_all_prims_conditional(bool_map, condition)
            }
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_all_conditional("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<bool>,
    {
        match self.remove_bool_conditional(field, condition) {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn remove_all_conditional<'a, T, R>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool>,
    {
        self.remove_all_bool_conditional(field, condition)
            .into_iter()
            .filter_map(|arc| match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            })
            .collect()
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

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct CompoundStruct {
        person: TestStruct,
    }

    #[test]
    fn get_conditional() {
        let mut int_entry = TreeSpaceEntry::new();
        assert_eq!(int_entry.get_conditional::<i64, _>("", 2..4), None);
        int_entry.add(3);
        int_entry.add(5);
        assert_eq!(int_entry.get_conditional::<i64, _>("", 2..4), Some(3));
        assert_ne!(int_entry.get_conditional::<i64, _>("", 2..4), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            test_struct_entry.get_conditional::<TestStruct, _>("count", 2..4),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(
            test_struct_entry
                .get_conditional::<TestStruct, _>("count", 2..4)
                .is_some()
        );

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });

        assert_eq!(
            compound_struct_entry.get_conditional::<CompoundStruct, _>("person.count", 2..4),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            compound_struct_entry
                .get_conditional::<CompoundStruct, _>("person.count", 2..4)
                .is_some()
        );
    }

    #[test]
    fn remove_conditional() {
        let mut int_entry = TreeSpaceEntry::new();
        assert_eq!(int_entry.remove_conditional::<i64, _>("", 2..4), None);
        int_entry.add(3);
        int_entry.add(5);
        assert_eq!(int_entry.remove_conditional::<i64, _>("", 2..4), Some(3));
        assert_eq!(int_entry.remove_conditional::<i64, _>("", 2..4), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            test_struct_entry.remove_conditional::<TestStruct, _>("count", 2..4),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(
            test_struct_entry
                .remove_conditional::<TestStruct, _>("count", 2..4)
                .is_none()
        );

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });

        assert_eq!(
            compound_struct_entry.remove_conditional::<CompoundStruct, _>("person.count", 2..4),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            compound_struct_entry
                .remove_conditional::<CompoundStruct, _>("person.count", 2..4)
                .is_none()
        );
    }

    #[test]
    fn get_all_conditional() {
        let mut int_entry = TreeSpaceEntry::new();
        int_entry.add(3);
        int_entry.add(5);
        assert_eq!(int_entry.get_all_conditional::<i64, _>("", 2..4).count(), 1);
        assert_eq!(int_entry.get_all_conditional::<i64, _>("", 2..4).count(), 1);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Minh"),
        });

        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            test_struct_entry
                .get_all_conditional::<TestStruct, _>("count", 2..4)
                .count(),
            2
        );
        assert_eq!(
            test_struct_entry
                .get_all_conditional::<TestStruct, _>("count", 2..4)
                .count(),
            2
        );

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Minh"),
            },
        });

        assert_eq!(
            compound_struct_entry
                .get_all_conditional::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            2
        );
        assert_eq!(
            compound_struct_entry
                .get_all_conditional::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            2
        );
    }

    #[test]
    fn remove_all_conditional() {
        let mut int_entry = TreeSpaceEntry::new();
        int_entry.add(3);
        int_entry.add(5);
        assert_eq!(
            int_entry.remove_all_conditional::<i64, _>("", 2..4).len(),
            1
        );
        assert_eq!(
            int_entry.remove_all_conditional::<i64, _>("", 2..4).len(),
            0
        );

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Minh"),
        });

        test_struct_entry.add(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            test_struct_entry
                .remove_all_conditional::<TestStruct, _>("count", 2..4)
                .len(),
            2
        );
        assert_eq!(
            test_struct_entry
                .remove_all_conditional::<TestStruct, _>("count", 2..4)
                .len(),
            0
        );
        assert_eq!(
            test_struct_entry
                .remove_all_conditional::<TestStruct, _>("count", 4..)
                .len(),
            1
        );

        let mut compound_struct_entry = TreeSpaceEntry::new();
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        compound_struct_entry.add(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Minh"),
            },
        });

        assert_eq!(
            compound_struct_entry
                .remove_all_conditional::<CompoundStruct, _>("person.count", 2..4)
                .len(),
            2
        );
        assert_eq!(
            compound_struct_entry
                .remove_all_conditional::<CompoundStruct, _>("person.count", 2..4)
                .len(),
            0
        );
        assert_eq!(
            compound_struct_entry
                .remove_all_conditional::<CompoundStruct, _>("person.count", 4..)
                .len(),
            1
        );
    }
}
