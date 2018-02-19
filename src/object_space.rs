use std::any::TypeId;
use std::iter;
use std::collections::range::RangeArgument;

use serde::{Deserialize, Serialize};
use ordered_float::NotNaN;
use chashmap::{CHashMap, ReadGuard, WriteGuard};

use entry::TreeSpaceEntry;
use entry::RangeEntry;
use entry::ExactKeyEntry;

pub trait ObjectSpaceRange<U>: ObjectSpace {
    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// return copies of all structs whose specified element is within the range.
    fn read_all_range<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_range<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return all structs whose specified element is within the range.
    fn take_all_range<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;
}

pub trait ObjectSpaceKey<U>: ObjectSpace {
    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_key<T, R>(&self, field: &str, key: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: Into<U> + Copy;

    /// Given a path to an element of the struct and a possible value,
    /// return copies of all structs whose specified element of the specified value.
    fn read_all_key<'a, T, R>(&'a self, field: &str, key: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: Into<U> + Copy;

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn read_key<T, R>(&self, field: &str, key: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: Into<U> + Copy;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_key<T, R>(&mut self, field: &str, key: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: Into<U> + Copy;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return all structs whose specified element of the specified value.
    fn take_all_key<'a, T, R>(&'a mut self, field: &str, key: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: Into<U> + Copy;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn take_key<T, R>(&mut self, field: &str, key: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: Into<U> + Copy;
}

pub trait ObjectSpace {
    /// Add a struct to the object space
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// return a copy of a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// return copies of all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// return a copy of a struct of type T
    /// The operation blocks until such a struct is found.
    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// remove and return a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// remove and return all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// remove and return a struct of type T
    /// The operation blocks until such a struct is found.
    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;
}

pub struct TreeObjectSpace {
    typeid_entries_dict: CHashMap<TypeId, TreeSpaceEntry>,
}

impl TreeObjectSpace {
    pub fn new() -> TreeObjectSpace {
        TreeObjectSpace {
            typeid_entries_dict: CHashMap::new(),
        }
    }

    fn get_object_entry_ref<T>(&self) -> Option<ReadGuard<TypeId, TreeSpaceEntry>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get(&type_id)
    }

    fn get_object_entry_mut<T>(&mut self) -> Option<WriteGuard<TypeId, TreeSpaceEntry>>
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
            .upsert(type_id, || default_entry, |_| ());
        self.typeid_entries_dict
            .get_mut(&type_id)
            .map(|mut guard| guard.add(obj));
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
            Some(ent) => Box::new(ent.get_all::<T>().collect::<Vec<T>>().into_iter()),
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
            Some(mut ent) => ent.remove::<T>(),
            None => None,
        }
    }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(mut entry) => Box::new(entry.remove_all::<T>().into_iter()),
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

macro_rules! object_range{
    ($($ty:ident)*) => {
        $(
            impl ObjectSpaceRange<$ty> for TreeObjectSpace {
                fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    match self.get_object_entry_ref::<T>() {
                        Some(entry) => entry.get_range::<T, _>(field, condition),
                        _ => None,
                    }
                }

                fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    match self.get_object_entry_ref::<T>() {
                        Some(ent) => Box::new(
                            ent.get_all_range::<T, _>(field, condition)
                                .collect::<Vec<T>>()
                                .into_iter(),
                        ),
                        None => Box::new(iter::empty::<T>()),
                    }
                }

                fn read_range<T, R>(&self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
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
                    R: RangeArgument<$ty> + Clone,
                {
                    match self.get_object_entry_mut::<T>() {
                        Some(mut entry) => entry.remove_range::<T, _>(field, condition),
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
                    R: RangeArgument<$ty> + Clone,
                {
                    match self.get_object_entry_mut::<T>() {
                        Some(mut ent) => Box::new(ent.remove_all_range::<T, _>(field, condition).into_iter()),
                        None => Box::new(iter::empty::<T>()),
                    }
                }

                fn take_range<T, R>(&mut self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    loop {
                        if let Some(item) = self.try_read_range::<T, _>(field, condition.clone()) {
                            return item;
                        }
                    }
                }
            }
        )*
    };
}

macro_rules! object_key{
    ($($ty:ty)*) => {
        $(
            impl ObjectSpaceKey<$ty> for TreeObjectSpace {
                fn try_read_key<T, R>(&self, field: &str, condition: R) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    match self.get_object_entry_ref::<T>() {
                        Some(entry) => entry.get_key::<T, _>(field, condition),
                        _ => None,
                    }
                }

                fn read_all_key<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    match self.get_object_entry_ref::<T>() {
                        Some(ent) => Box::new(ent.get_all_key::<T, _>(field, condition).collect::<Vec<T>>().into_iter()),
                        None => Box::new(iter::empty::<T>()),
                    }
                }

                fn read_key<T, R>(&self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    loop {
                        if let Some(item) = self.try_read_key::<T, _>(field, condition) {
                            return item;
                        }
                    }
                }

                fn try_take_key<T, R>(&mut self, field: &str, condition: R) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    match self.get_object_entry_mut::<T>() {
                        Some(mut entry) => entry.remove_key::<T, _>(field, condition),
                        _ => None,
                    }
                }

                fn take_all_key<'a, T, R>(
                    &'a mut self,
                    field: &str,
                    condition: R,
                ) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    match self.get_object_entry_mut::<T>() {
                        Some(mut ent) => Box::new(ent.remove_all_key::<T, _>(field, condition).into_iter()),
                        None => Box::new(iter::empty::<T>()),
                    }
                }

                fn take_key<T, R>(&mut self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: Into<$ty> + Copy,
                {
                    loop {
                        if let Some(item) = self.try_read_key::<T, _>(field, condition) {
                            return item;
                        }
                    }
                }
            }
        )*
    };
}

object_range!{i64 String bool}
object_key!{i64 String bool NotNaN<f64>}

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
