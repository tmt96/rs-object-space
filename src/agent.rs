use std::thread;
use std::thread::JoinHandle;
use serde::{Deserialize, Serialize};
use object_space::{ObjectSpace, ObjectSpaceKey, ObjectSpaceRange};
use std::collections::range::RangeArgument;

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
        for<'de> T: Serialize + Deserialize<'de> + 'static,
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

trait AgentRange<U> {
    type Space: ObjectSpaceRange<U>;

    fn get_space(&self) -> &Self::Space;

    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
    {
        self.get_space().try_read_range::<T, R>(field, condition)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// return copies of all structs whose specified element is within the range.
    fn read_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
        <Self as AgentRange<U>>::Space: 'a,
    {
        self.get_space().read_all_range::<T, R>(field, condition)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// return a copy of a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn read_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
    {
        self.get_space().read_range::<T, R>(field, condition)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_range<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
    {
        self.get_space().try_take_range::<T, R>(field, condition)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return all structs whose specified element is within the range.
    fn take_all_range<'a, T, R>(&'a self, field: &str, condition: R) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
        <Self as AgentRange<U>>::Space: 'a,
    {
        self.get_space().take_all_range::<T, R>(field, condition)
    }

    /// Given a path to an element of the struct and a range of possible values,
    /// remove and return a struct whose specified element is within the range.
    /// The operation blocks until a struct satisfies the condition is found.
    fn take_range<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone,
    {
        self.get_space().take_range::<T, R>(field, condition)
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
        for<'de> T: Serialize + Deserialize<'de> + 'static,
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
        <Self as AgentRange<U>>::Space: 'a,
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
        <Self as AgentRange<U>>::Space: 'a,
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

trait AgentKey<U> {
    type Space: ObjectSpaceKey<U>;

    fn get_space(&self) -> &Self::Space;

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_read_key<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_read_key::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// return copies of all structs whose specified element of the specified value.
    fn read_all_key<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        <Self as AgentKey<U>>::Space: 'a,
    {
        self.get_space().read_all_key::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// return a copy of a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn read_key<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().read_key::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is non-blocking and will returns None if no struct satisfies condition.
    fn try_take_key<T>(&self, field: &str, key: &U) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().try_take_key::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return all structs whose specified element of the specified value.
    fn take_all_key<'a, T>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        <Self as AgentKey<U>>::Space: 'a,
    {
        self.get_space().take_all_key::<T>(field, key)
    }

    /// Given a path to an element of the struct and a possible value,
    /// remove and return a struct whose specified element of the specified value.
    /// The operation is blocks until an element satisfies the condition is found.
    fn take_key<T>(&self, field: &str, key: &U) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        self.get_space().take_key::<T>(field, key)
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
        for<'de> T: Serialize + Deserialize<'de> + 'static,
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
        <Self as AgentKey<U>>::Space: 'a,
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
        <Self as AgentKey<U>>::Space: 'a,
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
