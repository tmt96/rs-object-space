use std::any::Any;
use std::marker::PhantomData;

pub trait TypeFamily {
    fn is_type<U>(&self, t: &Type<U>) -> bool
    where
        U: Default + Any;
    fn is_subtype_of<U>(&self, t: &Type<U>) -> bool
    where
        U: Default + Any;
    fn is_supertype_of<U>(&self, t: &Type<U>) -> bool
    where
        U: Default + Any;
}

pub struct Type<T: Default + Any> {
    marker: PhantomData<T>,
}

impl<T> TypeFamily for Type<T>
where
    T: Default + Any,
{
    fn is_type<U>(&self, _: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        let obj: &Any = &T::default();
        obj.is::<U>()
    }

    fn is_subtype_of<U>(&self, _: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        let obj: &Any = &T::default();
        obj.downcast_ref::<U>().is_some()
    }

    fn is_supertype_of<U>(&self, t: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        t.is_subtype_of::<T>(self)
    }
}
