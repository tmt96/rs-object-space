use std::any::TypeId;
use std::collections::range::RangeArgument;
use std::sync::{Arc, Condvar, Mutex};

use chashmap::{CHashMap, ReadGuard, WriteGuard};
use ordered_float::NotNaN;
use serde::{Deserialize, Serialize};
use serde_json::value::{from_value, to_value};

use entry::helpers::{deflatten, flatten};
use entry::{ExactKeyEntry, RangeEntry, TreeSpaceEntry};

/// Basic interface of an ObjectSpace.
///
/// This trait includes pushing, reading, and popping structs from the space.
/// An implementation of ObjectSpace should be thread-safe for usage in concurrent programs.
///
/// # Example
///
/// ```
/// # use object_space::{TreeObjectSpace, ObjectSpace};
/// let space = TreeObjectSpace::new();
/// space.write(String::from("Hello World"));
/// assert_eq!(
///     space.try_read::<String>(),
///     Some(String::from("Hello World"))
/// );
/// ```
pub trait ObjectSpace {
    /// Add a struct to the object space.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// space.write(String::from("Hello World"));
    /// ```
    fn write<T>(&self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Return a copy of a struct of type T.
    /// The operation is non-blocking
    /// and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// space.write(String::from("Hello World"));
    /// assert_eq!(
    ///     space.try_read::<String>(),
    ///     Some(String::from("Hello World"))
    /// );
    /// ```
    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Return copies of all structs of type T.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// assert_eq!(space.read_all::<String>().count(), 0);
    /// space.write("Hello".to_string());
    /// space.write("World".to_string());
    ///
    /// assert_eq!(
    ///     space.read_all::<String>().collect::<Vec<String>>(),
    ///     vec!["Hello", "World"]
    /// );
    /// ```
    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Return a copy of a struct of type T.
    /// The operation blocks until such a struct is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// space.write(String::from("Hello World"));
    /// assert_eq!(
    ///     space.read::<String>(),
    ///     String::from("Hello World")
    /// );
    /// ```
    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Remove and return a struct of type T.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// space.write(String::from("Hello World"));
    /// assert_eq!(
    ///     space.try_take::<String>(),
    ///     Some(String::from("Hello World"))
    /// );
    /// assert_eq!(space.try_take::<String>(), None);
    /// ```
    fn try_take<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Remove and return all structs of type T.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// assert_eq!(space.take_all::<String>().count(), 0);
    /// space.write("Hello".to_string());
    /// space.write("World".to_string());
    ///
    /// assert_eq!(
    ///     space.take_all::<String>().collect::<Vec<String>>(),
    ///     vec!["Hello", "World"]
    /// );
    /// assert_eq!(space.take_all::<String>().count(), 0);
    /// ```
    fn take_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Remove and return a struct of type T.
    /// The operation blocks until such a struct is found.
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace};
    /// let space = TreeObjectSpace::new();
    /// space.write(String::from("Hello World"));
    /// assert_eq!(
    ///     space.take::<String>(),
    ///     String::from("Hello World")
    /// );
    /// assert_eq!(space.try_take::<String>(), None);
    /// ```
    fn take<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;
}

/// An extension of `ObjectSpace` supporting retrieving structs by range of a field.
///
/// Given a type `T` with a field (might be nested) of type `U`,
/// a path to a field of type `U` and a `RangeArgument<U>`,
/// an `ObjectSpaceRange<U>` could retrieve structs of type `T`
/// whose value of the specified field is within the given range.
///
/// # Example
///
/// ```
/// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
/// let space = TreeObjectSpace::new();
/// space.write::<i64>(3);
/// space.write::<i64>(5);
///
/// assert_eq!(space.try_read_range::<i64, _>("", 2..4), Some(3));
/// assert_eq!(space.try_read_range::<i64, _>("", ..2), None);
/// ```
pub trait ObjectSpaceRange<U>: ObjectSpace {
    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.try_read_range::<i64, _>("", 2..4), Some(3));
    /// assert_eq!(space.try_read_range::<i64, _>("", ..2), None);
    /// ```
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// return copies of all structs whose specified element is within the range.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.read_all_range::<i64, _>("", 2..4).count(), 1);
    /// assert_eq!(space.read_all_range::<i64, _>("", 2..).count(), 2);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.read_range::<i64, _>("", 2..4), 3);
    /// ```
    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.try_take_range::<i64, _>("", 2..4), Some(3));
    /// assert_eq!(space.try_take_range::<i64, _>("", 2..4), None);
    /// assert_eq!(space.try_take_range::<i64, _>("", 2..), Some(5));
    /// ```
    fn try_take_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return all structs whose specified element is within the range.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.take_all_range::<i64, _>("", 2..4).count(), 1);
    /// assert_eq!(space.take_all_range::<i64, _>("", 2..).count(), 1);
    /// ```
    fn take_all_range<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceRange};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.take_range::<i64, _>("", 2..4), 3);
    /// assert_eq!(space.take_range::<i64, _>("", 2..), 5);
    /// ```
    fn take_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;
}

/// An extension of `ObjectSpace` supporting retrieving structs by value of a field.
///
/// Given a type `T` with a field (might be nested) of type `U`,
/// a path to a field of type `U` and a value of type `U`,
/// an `ObjectSpaceKey<U>` could retrieve structs of type `T`
/// whose value of the specified field equals to the specified value.
///
/// # Example
///
/// ```
/// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
/// let space = TreeObjectSpace::new();
/// space.write::<i64>(3);
/// space.write::<i64>(5);
///
/// assert_eq!(space.try_read_key::<i64>("", &3), Some(3));
/// assert_eq!(space.try_read_key::<i64>("", &2), None);
/// ```
pub trait ObjectSpaceKey<U>: ObjectSpace {
    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.try_read_key::<i64>("", &3), Some(3));
    /// assert_eq!(space.try_read_key::<i64>("", &2), None);
    /// ```
    fn try_read_key<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Given a path to an element of the struct and a possible value,
    /// return copies of all structs whose specified element of the specified value.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.read_all_key::<i64>("", &3).count(), 1);
    /// assert_eq!(space.read_all_key::<i64>("", &2).count(), 0);
    /// ```
    fn read_all_key<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static;

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.read_key::<i64>("", &3), 3);
    /// ```
    fn read_key<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.try_take_key::<i64>("", &3), Some(3));
    /// assert_eq!(space.try_take_key::<i64>("", &3), None);
    /// assert_eq!(space.try_take_key::<i64>("", &4), None);
    /// ```
    fn try_take_key<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return all structs whose specified element of the specified value.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.take_all_key::<i64>("", &3).count(), 1);
    /// assert_eq!(space.take_all_key::<i64>("", &4).count(), 0);
    /// ```
    fn take_all_key<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static;

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use object_space::{TreeObjectSpace, ObjectSpace, ObjectSpaceKey};
    /// let space = TreeObjectSpace::new();
    /// space.write::<i64>(3);
    /// space.write::<i64>(5);
    ///
    /// assert_eq!(space.take_key::<i64>("", &3), 3);
    /// ```
    fn take_key<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;
}

type Lock = Arc<(Mutex<bool>, Condvar)>;

/// A thread-safe reference `ObjectSpace` implementation
///
/// # Implementation
///
/// A `TreeObjectSpace` is a `HashMap` between a `TypeId`
/// and the actual `Entry` structure holding the structs.
/// Before structs are stored in `Entry`,
/// they are serialized into a JSON-like structure and then flattened.
///
/// An `Entry` is a `HashMap` whose key is a flattened field and
/// value is a `BTreeMap` between possible values of the field
/// and the `Vec` of structs containing the corresponding value of such field.
///
/// `Mutex` is used sparingly to ensure blocking `read` and `take` calls do not hijack CPU cycles
pub struct TreeObjectSpace {
    typeid_entries_dict: CHashMap<TypeId, TreeSpaceEntry>,
    lock_dict: CHashMap<TypeId, Lock>,
}

impl TreeObjectSpace {
    pub fn new() -> TreeObjectSpace {
        TreeObjectSpace {
            typeid_entries_dict: CHashMap::new(),
            lock_dict: CHashMap::new(),
        }
    }

    fn get_object_entry_ref<'a, T>(&'a self) -> Option<ReadGuard<TypeId, TreeSpaceEntry>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get(&type_id)
    }

    fn get_object_entry_mut<T>(&self) -> Option<WriteGuard<TypeId, TreeSpaceEntry>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get_mut(&type_id)
    }

    fn get_lock<T>(&self) -> Option<ReadGuard<TypeId, Lock>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.lock_dict.get(&type_id)
    }

    fn add_entry(&self, id: TypeId) {
        let default_value = TreeSpaceEntry::new();

        self.typeid_entries_dict
            .upsert(id, || default_value, |_| ());
        self.lock_dict
            .upsert(id, || Arc::new((Mutex::new(false), Condvar::new())), |_| ());
    }
}

impl ObjectSpace for TreeObjectSpace {
    fn write<T>(&self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.add_entry(type_id);
        let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
        let value = flatten(to_value(obj).expect("struct cannot be serialized"));
        let mut status = lock.lock().unwrap();
        *status = !*status;
        self.typeid_entries_dict
            .get_mut(&type_id)
            .unwrap()
            .add(value);
        cvar.notify_all();
    }

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let value = match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get(),
            _ => None,
        };
        match value {
            Some(val) => from_value(deflatten(val)).ok(),
            _ => None,
        }
    }

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let val_iter: Vec<_> = match self.get_object_entry_ref::<T>() {
            Some(ent) => ent.get_all().collect(),
            None => Vec::new(),
        };

        Box::new(
            val_iter
                .into_iter()
                .filter_map(|item| from_value(deflatten(item)).ok()),
        )
    }

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.add_entry(TypeId::of::<T>());
        let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
        let value;
        {
            let mut fetched = lock.lock().unwrap();
            loop {
                let result = match self.get_object_entry_ref::<T>() {
                    Some(entry) => entry.get(),
                    _ => None,
                };
                if let Some(item) = result {
                    value = item;
                    break;
                }
                fetched = cvar.wait(fetched).unwrap();
            }
        }
        from_value(deflatten(value)).unwrap()
    }

    fn try_take<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let value = match self.get_object_entry_mut::<T>() {
            Some(mut entry) => entry.remove(),
            _ => None,
        };
        match value {
            Some(val) => from_value(deflatten(val)).ok(),
            _ => None,
        }
    }

    fn take_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let val_iter = match self.get_object_entry_mut::<T>() {
            Some(mut ent) => ent.remove_all(),
            None => Vec::new(),
        };

        Box::new(
            val_iter
                .into_iter()
                .filter_map(|item| from_value(deflatten(item)).ok()),
        )
    }

    fn take<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.add_entry(TypeId::of::<T>());
        let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
        let value;
        {
            let mut fetched = lock.lock().unwrap();
            loop {
                let result = match self.get_object_entry_mut::<T>() {
                    Some(mut entry) => entry.remove(),
                    _ => None,
                };
                if let Some(item) = result {
                    value = item;
                    break;
                }
                fetched = cvar.wait(fetched).unwrap();
            }
        }
        from_value(deflatten(value)).unwrap()
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
                    let value = match self.get_object_entry_ref::<T>() {
                        Some(entry) => entry.get_range::<_>(field, condition),
                        _ => None,
                    };
                    match value {
                        Some(val) => from_value(deflatten(val)).ok(),
                        _ => None,
                    }
                }

                fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    let val_iter: Vec<_> = match self.get_object_entry_ref::<T>() {
                        Some(ent) => ent.get_all_range::<_>(field, condition).collect(),
                        None => Vec::new(),
                    };

                    Box::new(val_iter.into_iter().filter_map(|item| from_value(deflatten(item)).ok()))
                }

                fn read_range<T, R>(&self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    self.add_entry(TypeId::of::<T>());
                    let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
                    let value;
                    {
                        let mut fetched = lock.lock().unwrap();
                        loop {
                            let result = match self.get_object_entry_ref::<T>() {
                                Some(entry) => entry.get_range::<_>(field, condition.clone()),
                                _ => None,
                            };
                            if let Some(item) = result {
                                value = item;
                                break;
                            }
                            fetched = cvar.wait(fetched).unwrap();
                        }
                    }
                    from_value(deflatten(value)).unwrap()
                }

                fn try_take_range<T, R>(&self, field: &str, condition: R) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    let value = match self.get_object_entry_mut::<T>() {
                        Some(mut entry) => entry.remove_range::<_>(field, condition),
                        _ => None,
                    };
                    match value {
                        Some(val) => from_value(deflatten(val)).ok(),
                        _ => None,
                    }
                }

                fn take_all_range<'a, T, R>(
                    &'a self,
                    field: &str,
                    condition: R,
                ) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    let val_iter = match self.get_object_entry_mut::<T>() {
                        Some(mut ent) => ent.remove_all_range::<_>(field, condition),
                        None => Vec::new(),
                    };

                    Box::new(
                        val_iter
                            .into_iter()
                            .filter_map(|item| from_value(deflatten(item)).ok())
                    )
                }

                fn take_range<T, R>(&self, field: &str, condition: R) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                    R: RangeArgument<$ty> + Clone,
                {
                    self.add_entry(TypeId::of::<T>());
                    let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
                    let value;
                    {
                        let mut fetched = lock.lock().unwrap();
                        loop {
                            let result = match self.get_object_entry_mut::<T>() {
                                Some(mut entry) => entry.remove_range::<_>(field, condition.clone()),
                                _ => None,
                            };
                            if let Some(item) = result {
                                value = item;
                                break;
                            }
                            fetched = cvar.wait(fetched).unwrap();
                        }
                    }
                    from_value(deflatten(value)).unwrap()
                }
            }
        )*
    };
}

macro_rules! object_key{
    ($($ty:ty)*) => {
        $(
            impl ObjectSpaceKey<$ty> for TreeObjectSpace {
                fn try_read_key<T>(&self, field: &str, condition: &$ty) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                {
                    let value = match self.get_object_entry_ref::<T>() {
                        Some(entry) => entry.get_key(field, condition),
                        _ => None,
                    };
                    match value {
                        Some(val) => from_value(deflatten(val)).ok(),
                        _ => None,
                    }
                }

                fn read_all_key<'a, T>(&'a self, field: &str, condition: &$ty) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                {
                    let val_iter: Vec<_> = match self.get_object_entry_ref::<T>() {
                        Some(ent) => ent.get_all_key(field, condition).collect(),
                        None => Vec::new(),
                    };

                    Box::new(val_iter.into_iter().filter_map(|item| from_value(deflatten(item)).ok()))
                }

                fn read_key<T>(&self, field: &str, condition: &$ty) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                {
                    self.add_entry(TypeId::of::<T>());
                    let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
                    let value;
                    {
                        let mut fetched = lock.lock().unwrap();
                        loop {
                            let result = match self.get_object_entry_ref::<T>() {
                                Some(entry) => entry.get_key(field, condition),
                                _ => None,
                            };
                            if let Some(item) = result {
                                value = item;
                                break;
                            }
                            fetched = cvar.wait(fetched).unwrap();
                        }
                    }
                    from_value(deflatten(value)).unwrap()
                }

                fn try_take_key<T>(&self, field: &str, condition: &$ty) -> Option<T>
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                {
                    let value = match self.get_object_entry_mut::<T>() {
                        Some(mut entry) => entry.remove_key(field, condition),
                        _ => None,
                    };
                    match value {
                        Some(val) => from_value(deflatten(val)).ok(),
                        _ => None,
                    }
                }

                fn take_all_key<'a, T>(
                    &'a self,
                    field: &str,
                    condition: &$ty,
                ) -> Box<Iterator<Item = T> + 'a>
                where
                    for<'de> T: Deserialize<'de> + 'static,
                {
                    let val_iter = match self.get_object_entry_mut::<T>() {
                        Some(mut ent) => ent.remove_all_key(field, condition),
                        None => Vec::new(),
                    };

                    Box::new(
                        val_iter
                            .into_iter()
                            .filter_map(|item| from_value(deflatten(item)).ok())
                    )
                }

                fn take_key<T>(&self, field: &str, condition: &$ty) -> T
                where
                    for<'de> T: Serialize + Deserialize<'de> + 'static,
                {
                    self.add_entry(TypeId::of::<T>());
                    let &(ref lock, ref cvar) = &*self.get_lock::<T>().unwrap().clone();
                    let value;
                    {
                        let mut fetched = lock.lock().unwrap();
                        loop {
                            let result = match self.get_object_entry_mut::<T>() {
                                Some(mut entry) => entry.remove_key(field, condition),
                                _ => None,
                            };
                            if let Some(item) = result {
                                value = item;
                                break;
                            }
                            fetched = cvar.wait(fetched).unwrap();
                        }
                    }
                    from_value(deflatten(value)).unwrap()
                }
            }
        )*
    };
}

object_range!{i64 String}
object_key!{i64 String bool f64}

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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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
        let space = TreeObjectSpace::new();
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

    #[test]
    fn try_read_key() {
        let space = TreeObjectSpace::new();
        assert_eq!(space.try_read_key::<i64>("", &3), None);
        space.write::<i64>(3);
        space.write::<i64>(5);

        assert_eq!(space.try_read_key::<i64>("", &3), Some(3));
        assert_eq!(space.try_read_key::<i64>("", &2), None);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        space.write(TestStruct {
            count: 5,
            name: String::from("Duane"),
        });

        assert_eq!(
            space.try_read_key::<TestStruct>("count", &3),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(space.try_read_key::<TestStruct>("count", &3).is_some());

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
            space.try_read_key::<CompoundStruct>("person.count", &3),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            space
                .try_read_key::<CompoundStruct>("person.count", &3)
                .is_some()
        );
    }

    #[test]
    fn try_take_key() {
        let space = TreeObjectSpace::new();
        assert_eq!(space.try_take_key::<i64>("", &3), None);
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.try_take_key::<i64>("", &4), None);
        assert_eq!(space.try_take_key::<i64>("", &3), Some(3));
        assert_eq!(space.try_take_key::<i64>("", &3), None);

        space.write(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        assert_eq!(
            space.try_take_key::<TestStruct>("count", &3),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert!(space.try_take_key::<TestStruct>("count", &3).is_none());

        space.write(CompoundStruct {
            person: TestStruct {
                count: 3,
                name: String::from("Tuan"),
            },
        });

        assert_eq!(
            space.try_take_key::<CompoundStruct>("person.count", &3),
            Some(CompoundStruct {
                person: TestStruct {
                    count: 3,
                    name: String::from("Tuan"),
                },
            })
        );
        assert!(
            space
                .try_take_key::<CompoundStruct>("person.count", &3)
                .is_none()
        );
    }

    #[test]
    fn read_all_key() {
        let space = TreeObjectSpace::new();
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.read_all_key::<i64>("", &3).count(), 1);
        assert_eq!(space.read_all_key::<i64>("", &4).count(), 0);

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

        assert_eq!(space.read_all_key::<TestStruct>("count", &3).count(), 2);
        assert_eq!(space.read_all_key::<TestStruct>("count", &4).count(), 0);

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
                .read_all_key::<CompoundStruct>("person.count", &3)
                .count(),
            2
        );
        assert_eq!(
            space
                .read_all_key::<CompoundStruct>("person.count", &4)
                .count(),
            0
        );
        assert_eq!(
            space
                .read_all_key::<CompoundStruct>("person.count", &5)
                .count(),
            1
        );
    }

    #[test]
    fn take_all_key() {
        let space = TreeObjectSpace::new();
        space.write::<i64>(3);
        space.write::<i64>(5);
        assert_eq!(space.take_all_key::<i64>("", &3).count(), 1);
        assert_eq!(space.take_all_key::<i64>("", &4).count(), 0);

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

        assert_eq!(space.take_all_key::<TestStruct>("count", &3).count(), 2);
        assert_eq!(space.take_all_key::<TestStruct>("count", &3).count(), 0);
        assert_eq!(space.take_all_key::<TestStruct>("count", &5).count(), 1);

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
                .take_all_key::<CompoundStruct>("person.count", &3)
                .count(),
            2
        );
        assert_eq!(
            space
                .take_all_key::<CompoundStruct>("person.count", &3)
                .count(),
            0
        );
        assert_eq!(
            space
                .take_all_key::<CompoundStruct>("person.count", &5)
                .count(),
            1
        );
    }
}
