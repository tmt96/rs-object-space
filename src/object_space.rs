use std::collections::HashMap;
use std::any::TypeId;
use std::iter;
use std::collections::range::RangeArgument;
use serde::{Deserialize, Serialize};

use entry::TreeSpaceEntry;
use entry::RangeEntry;

pub trait ObjectSpacerange<U>: ObjectSpace {
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn read_all_range<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn try_take_range<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn take_all_range<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;
}

pub trait ObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;
}

pub struct TreeObjectSpace {
    typeid_entries_dict: HashMap<TypeId, TreeSpaceEntry>,
}

impl TreeObjectSpace {
    pub fn new() -> TreeObjectSpace {
        TreeObjectSpace {
            typeid_entries_dict: HashMap::new(),
        }
    }

    fn get_object_entry_ref<T>(&self) -> Option<&TreeSpaceEntry>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get(&type_id)
    }

    fn get_object_entry_mut<T>(&mut self) -> Option<&mut TreeSpaceEntry>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get_mut(&type_id)
    }
}

impl ObjectSpace for TreeObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let default_entry = TreeSpaceEntry::new();
        let type_id = TypeId::of::<T>();

        self.typeid_entries_dict
            .entry(type_id)
            .or_insert(default_entry)
            .add(obj);
    }

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get::<T>(),
            _ => None,
        }
    }

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all::<T>()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        loop {
            if let Some(item) = self.try_read::<T>() {
                return item;
            }
        }
    }

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => ent.remove::<T>(),
            None => None,
        }
    }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => Box::new(entry.remove_all::<T>().into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        loop {
            if let Some(item) = self.try_take::<T>() {
                return item;
            }
        }
    }
}

impl ObjectSpacerange<i64> for TreeObjectSpace {
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_range::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_range<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_range<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(ent.remove_all_range::<T, _>(field, condition).into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}

impl ObjectSpacerange<String> for TreeObjectSpace {
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_range::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_range<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_range<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(ent.remove_all_range::<T, _>(field, condition).into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}

impl ObjectSpacerange<bool> for TreeObjectSpace {
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_range::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_range<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_range::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_range<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(ent.remove_all_range::<T, _>(field, condition).into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}

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
    fn try_read() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.try_read::<String>(), None);
        space.write(String::from("Hello World"));
        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });

        assert_eq!(
            space.try_read::<String>(),
            Some(String::from("Hello World"))
        );
        assert_ne!(space.try_read::<String>(), None);

        assert_eq!(
            space.try_read::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );

        assert_eq!(
            space.try_read::<CompoundStruct>(),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(space.try_read::<CompoundStruct>().is_some());
    }

    #[test]
    fn try_take() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.try_take::<String>(), None);
        space.write(String::from("Hello World"));
        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });

        assert_eq!(
            space.try_take::<String>(),
            Some(String::from("Hello World"))
        );
        assert_eq!(space.try_take::<String>(), None);

        assert_eq!(
            space.try_take::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert_eq!(space.try_take::<TestStruct>(), None);

        assert_eq!(
            space.try_take::<CompoundStruct>(),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(space.try_take::<CompoundStruct>().is_none());
    }

    #[test]
    fn read_all() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.read_all::<String>().count(), 0);
        space.write("Hello".to_string());
        space.write("World".to_string());
        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.read_all::<String>().collect::<Vec<String>>(),
            vec!["Hello", "World"]
        );
        assert_ne!(space.read_all::<String>().count(), 0);

        assert_eq!(space.read_all::<TestStruct>().count(), 2);
        assert_eq!(space.read_all::<TestStruct>().count(), 2);
    }

    #[test]
    fn take_all() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.take_all::<String>().count(), 0);
        space.write("Hello".to_string());
        space.write("World".to_string());
        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(space.take_all::<String>().count(), 2);
        assert_eq!(space.take_all::<String>().count(), 0);

        assert_eq!(space.take_all::<TestStruct>().count(), 2);
        assert_eq!(space.take_all::<TestStruct>().count(), 0);
    }

    #[test]
    fn try_read_range() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.try_read_range::<i64, _>("", 2..4), None);
        space.write::<i64>(3);
        space.write::<i64>(5);

        assert_eq!(space.try_read_range::<i64, _>("", 2..4), Some(3));
        assert_ne!(space.try_read_range::<i64, _>("", 2..4), None);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.try_read_range::<TestStruct, _>("count", 2..4),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(
            space
                .try_read_range::<TestStruct, _>("count", 2..4)
                .is_some()
        );

        space.write(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });

        assert_eq!(
            space.try_read_range::<CompoundStruct, _>("person.count", 2..4),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            space
                .try_read_range::<CompoundStruct, _>("person.count", 2..4)
                .is_some()
        );
    }

    #[test]
    fn try_take_range() {
        let mut space = TreeObjectSpace::new();
        assert_eq!(space.try_take_range::<i64, _>("", 2..4), None);
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.try_take_range::<i64, _>("", 2..4), Some(3));
        assert_eq!(space.try_take_range::<i64, _>("", 2..4), None);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.try_take_range::<TestStruct, _>("count", 2..4),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(
            space
                .try_take_range::<TestStruct, _>("count", 2..4)
                .is_none()
        );

        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });

        assert_eq!(
            space.try_take_range::<CompoundStruct, _>("person.count", 2..4),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            space
                .try_take_range::<CompoundStruct, _>("person.count", 2..4)
                .is_none()
        );
    }

    #[test]
    fn read_all_range() {
        let mut space = TreeObjectSpace::new();
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.read_all_range::<i64, _>("", 2..4).count(), 1);
        assert_eq!(space.read_all_range::<i64, _>("", 2..4).count(), 1);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 3,
            name: String::from("Minh"),
        });

        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.read_all_range::<TestStruct, _>("count", 2..4).count(),
            2
        );
        assert_eq!(
            space.read_all_range::<TestStruct, _>("count", 2..4).count(),
            2
        );

        space.write(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Minh"),
            },
        });

        assert_eq!(
            space
                .read_all_range::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            2
        );
        assert_eq!(
            space
                .read_all_range::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            2
        );
    }

    #[test]
    fn take_all_range() {
        let mut space = TreeObjectSpace::new();
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.take_all_range::<i64, _>("", 2..4).count(), 1);
        assert_eq!(space.take_all_range::<i64, _>("", 2..4).count(), 0);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 3,
            name: String::from("Minh"),
        });

        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.take_all_range::<TestStruct, _>("count", 2..4).count(),
            2
        );
        assert_eq!(
            space.take_all_range::<TestStruct, _>("count", 2..4).count(),
            0
        );
        assert_eq!(
            space.take_all_range::<TestStruct, _>("count", 4..).count(),
            1
        );

        space.write(CompoundStruct {
            person: TestStruct {
                count: 5,
                name: String::from("Duane"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });
        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Minh"),
            },
        });

        assert_eq!(
            space
                .take_all_range::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            2
        );
        assert_eq!(
            space
                .take_all_range::<CompoundStruct, _>("person.count", 2..4)
                .count(),
            0
        );
        assert_eq!(
            space
                .take_all_range::<CompoundStruct, _>("person.count", 4..)
                .count(),
            1
        );
    }

}
