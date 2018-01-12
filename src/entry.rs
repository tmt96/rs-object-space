use std::iter::Filter;
use std::vec::{Drain, DrainFilter};
use std::any::Any;
use std::slice::Iter;

pub trait ObjectSpaceEntryFamily {
    fn as_any_ref(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

pub struct ObjectSpaceEntry<T: Default + Any> {
    object_list: Vec<T>,
}

impl<T> ObjectSpaceEntryFamily for ObjectSpaceEntry<T>
where
    T: Default + Any,
{
    fn as_any_ref(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl<T> ObjectSpaceEntry<T>
where
    T: Default + Any,
{
    pub fn new() -> ObjectSpaceEntry<T> {
        ObjectSpaceEntry::<T> {
            object_list: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: T) {
        &self.object_list.push(obj);
    }

    pub fn get(&self) -> Option<&T> {
        self.object_list.first()
    }

    pub fn get_conditional<P>(&self, cond: &P) -> Option<&T>
    where
        P: Fn(&T) -> bool,
    {
        match self.object_list.iter().position(cond) {
            Some(index) => self.object_list.get(index),
            None => None,
        }
    }

    pub fn get_all(&self) -> Iter<T> {
        self.object_list.iter()
    }

    pub fn get_all_conditional<P>(&self, cond: P) -> Filter<Iter<T>, P>
    where
        for<'r> P: Fn(&'r &T) -> bool,
    {
        self.object_list.iter().filter(cond)
    }

    pub fn remove(&mut self) -> Option<T> {
        self.object_list.pop()
    }

    pub fn remove_conditional<P>(&mut self, cond: &P) -> Option<T>
    where
        P: Fn(&T) -> bool,
    {
        self.object_list
            .iter()
            .position(cond)
            .map(|index| self.object_list.remove(index))
    }

    pub fn remove_all(&mut self) -> Drain<T> {
        self.object_list.drain(..)
    }

    pub fn remove_all_conditional<P>(&mut self, cond: P) -> DrainFilter<T, P>
    where
        for<'r> P: Fn(&'r mut T) -> bool,
    {
        self.object_list.drain_filter(cond)
    }

    pub fn len(&self) -> usize {
        self.object_list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        let mut entry = ObjectSpaceEntry::<String>::new();
        assert_eq!(entry.get(), None);
        entry.add(String::from("Hello World"));
        assert_eq!(entry.get(), Some(&String::from("Hello World")));
        assert_ne!(entry.get(), None);
    }

    #[test]
    fn remove() {
        let mut entry = ObjectSpaceEntry::<String>::new();
        assert_eq!(entry.remove(), None);
        entry.add(String::from("Hello World"));
        assert_eq!(entry.remove(), Some(String::from("Hello World")));
        assert_eq!(entry.remove(), None);
    }

    #[test]
    fn get_all() {
        let mut entry = ObjectSpaceEntry::<String>::new();
        assert_eq!(entry.get_all().count(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        let mut iter = entry.get_all();
        assert_eq!(iter.next(), Some(&"Hello".to_string()));
        assert_eq!(iter.next(), Some(&"World".to_string()));
        assert_eq!(iter.next(), None);
        assert_ne!(entry.len(), 0);
    }

    #[test]
    fn remove_all() {
        let mut entry = ObjectSpaceEntry::<String>::new();
        assert_eq!(entry.remove_all().count(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        let mut iter = entry.remove_all();
        assert_eq!(iter.next(), Some("Hello".to_string()));
        assert_eq!(iter.next(), Some("World".to_string()));
        assert_eq!(iter.next(), None);
        // TODO: check for length
    }

}
