use std::any::Any;
use std::marker::PhantomData;

pub trait TypeFamily {
    fn is_type<U>(t: &Type<U>) -> bool
    where
        U: Default + Any;
    fn is_subtype_of<U>(t: &Type<U>) -> bool
    where
        U: Default + Any;
    fn is_supertype_of<U>(t: &Type<U>) -> bool
    where
        U: Default + Any;
}

#[derive(Hash, Eq)]
pub struct Type<T: Default + Any> {
    marker: PhantomData<T>,
}

impl<T> Type<T>
where
    T: Default + Any,
{
    fn is_type_helper<U>() -> bool
    where
        U: Default + Any,
    {
        let obj: &Any = &T::default();
        obj.is::<U>()
    }

    fn is_subtype_of_helper<U>() -> bool
    where
        U: Default + Any,
    {
        let obj: &Any = &T::default();
        obj.downcast_ref::<U>().is_some()
    }
}

impl<T, U> PartialEq<Type<U>> for Type<T>
where
    T: Default + Any,
    U: Default + Any,
{
    fn eq(&self, _: &Type<U>) -> bool {
        Self::is_type_helper::<U>()
    }
}

impl<T> TypeFamily for Type<T>
where
    T: Default + Any,
{
    fn is_type<U>(_: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        Self::is_type_helper::<U>()
    }

    fn is_subtype_of<U>(_: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        Self::is_subtype_of_helper::<U>()
    }

    fn is_supertype_of<U>(_: &Type<U>) -> bool
    where
        U: Default + Any,
    {
        Type::<U>::is_subtype_of_helper::<T>()
    }
}
