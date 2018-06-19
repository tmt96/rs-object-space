use object_space::{ObjectSpace, RangeLookupObjectSpace, ValueLookupObjectSpace};
use serde::{Deserialize, Serialize};
use std::ops::RangeBounds;
use std::thread;
use std::thread::JoinHandle;

pub trait Agent {
    type Space: ObjectSpace;

    fn get_space(&self) -> &Self::Space;

    fn start<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(f)
    }

    /// Add a struct to the object space
    fn write<T>(&self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + Send + 'static,
    {
        self.get_space().write(obj)
    }

    /// return a copy of a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_read::<T>()
    }

    /// return copies of all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read_all::<T>()
    }

    /// return a copy of a struct of type T
    /// The operation blocks until such a struct is found.
    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read::<T>()
    }

    /// remove and return a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_take::<T>()
    }

    /// remove and return all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn take_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take_all::<T>()
    }

    /// remove and return a struct of type T
    /// The operation blocks until such a struct is found.
    fn take<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take::<T>()
    }
}

pub trait RangeLookupAgent<U> {
    type Space: RangeLookupObjectSpace<U>;

    fn get_space(&self) -> &Self::Space;

    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_by_range<T, R>(&self, field: &str, range: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
    {
        self.get_space().try_read_by_range::<T, R>(field, range)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// return copies of all structs whose specified element is within the range.
    fn read_all_by_range<'a, T, R>(&'a self, field: &str, range: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
        <Self as RangeLookupAgent<U>>::Space: 'a,
    {
        self.get_space().read_all_by_range::<T, R>(field, range)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn read_by_range<T, R>(&self, field: &str, range: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
    {
        self.get_space().read_by_range::<T, R>(field, range)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_by_range<T, R>(&self, field: &str, range: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
    {
        self.get_space().try_take_by_range::<T, R>(field, range)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return all structs whose specified element is within the range.
    fn take_all_by_range<'a, T, R>(&'a self, field: &str, range: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
        <Self as RangeLookupAgent<U>>::Space: 'a,
    {
        self.get_space().take_all_by_range::<T, R>(field, range)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn take_by_range<T, R>(&self, field: &str, range: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeBounds<U> + Clone,
    {
        self.get_space().take_by_range::<T, R>(field, range)
    }

    fn start<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(f)
    }

    /// Add a struct to the object space
    fn write<T>(&self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + Send + 'static,
    {
        self.get_space().write(obj)
    }

    /// return a copy of a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_read::<T>()
    }

    /// return copies of all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        <Self as RangeLookupAgent<U>>::Space: 'a,
    {
        self.get_space().read_all::<T>()
    }

    /// return a copy of a struct of type T
    /// The operation blocks until such a struct is found.
    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read::<T>()
    }

    /// remove and return a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_take::<T>()
    }

    /// remove and return all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn take_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        <Self as RangeLookupAgent<U>>::Space: 'a,
    {
        self.get_space().take_all::<T>()
    }

    /// remove and return a struct of type T
    /// The operation blocks until such a struct is found.
    fn take<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take::<T>()
    }
}

pub trait ValueLookupAgent<U> {
    type Space: ValueLookupObjectSpace<U>;

    fn get_space(&self) -> &Self::Space;

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_by_value<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_read_by_value::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// return copies of all structs whose specified element of the specified value.
    fn read_all_by_value<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        <Self as ValueLookupAgent<U>>::Space: 'a,
    {
        self.get_space().read_all_by_value::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn read_by_value<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read_by_value::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_by_value<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_take_by_value::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return all structs whose specified element of the specified value.
    fn take_all_by_value<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        <Self as ValueLookupAgent<U>>::Space: 'a,
    {
        self.get_space().take_all_by_value::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn take_by_value<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take_by_value::<T>(field, key)
    }

    fn start<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(f)
    }

    /// Add a struct to the object space
    fn write<T>(&self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + Send + 'static,
    {
        self.get_space().write(obj)
    }

    /// return a copy of a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_read::<T>()
    }

    /// return copies of all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        <Self as ValueLookupAgent<U>>::Space: 'a,
    {
        self.get_space().read_all::<T>()
    }

    /// return a copy of a struct of type T
    /// The operation blocks until such a struct is found.
    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read::<T>()
    }

    /// remove and return a struct of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_take::<T>()
    }

    /// remove and return all structs of type T
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn take_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        <Self as ValueLookupAgent<U>>::Space: 'a,
    {
        self.get_space().take_all::<T>()
    }

    /// remove and return a struct of type T
    /// The operation blocks until such a struct is found.
    fn take<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take::<T>()
    }
}
