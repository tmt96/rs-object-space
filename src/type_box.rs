use std::any::{Any, TypeId};
use std::marker::Unsize;
use std::marker::PhantomData;

pub trait TypeFamily {
    fn as_any_ref(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

pub struct Type<T: Any> {
    obj: PhantomData<T>,
}

impl<T> Type<T>
where
    T: Any,
{
    pub fn new() -> Type<T> {
        Type { obj: PhantomData }
    }

    pub fn from_struct(_: &T) -> Type<T> {
        Type { obj: PhantomData }
    }

    fn is_type<U>(&self) -> bool
    where
        U: Any,
    {
        TypeId::of::<T>() == TypeId::of::<U>()
    }

    fn is_subtype_of<U>(&self) -> bool
    where
        U: ?Sized,
    {
        <T as ImplTrait<U>>::has_trait()
    }
}

impl<T> TypeFamily for Type<T>
where
    T: Any,
{
    fn as_any_ref(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

trait ImplTrait<TraitType: ?Sized> {
    fn has_trait() -> bool;
}

default impl<TraitType: ?Sized, T> ImplTrait<TraitType> for T {
    fn has_trait() -> bool {
        false
    }
}

impl<TraitType: ?Sized, T> ImplTrait<TraitType> for T
where
    T: Unsize<TraitType>,
{
    fn has_trait() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn is_type() {
        let string_type = Type::from_struct(&String::from("Hello World"));
        let str_type = Type::from_struct(&"Hello World");
        assert!(string_type.is_type::<String>());
        assert!(!str_type.is_type::<String>());
    }

    #[test]
    fn is_subtype_of() {
        let string = String::from("Hello World");
        let string_type = Type::from_struct(&string);
        assert!(string_type.is_subtype_of::<Write>());
    }
}
