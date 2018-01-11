use std::collections::HashMap;
use std::slice::Iter;
use std::any::{Any, TypeId};
use std::iter;
use std::vec::Drain;

pub trait ObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        T: Default + Any;

    fn read<T>(&self) -> Option<&T>
    where
        T: Default + Any;

    fn read_conditional<P, T>(&self, cond: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
        T: Default + Any;

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = &T> + 'a>
    where
        T: Default + Any;

    fn take<T>(&mut self) -> Option<T>
    where
        T: Default + Any;

    fn take_conditional<P, T>(&mut self, cond: P) -> Option<T>
    where
        P: FnMut(&T) -> bool,
        T: Default + Any;

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        T: Default + Any;
}

pub struct SequentialObjectSpace {
    typeid_entries_dict: HashMap<TypeId, Box<ObjectSpaceEntryFamily>>,
}

impl SequentialObjectSpace {
    pub fn new() -> SequentialObjectSpace {
        SequentialObjectSpace {
            typeid_entries_dict: HashMap::new(),
        }
    }
}

impl ObjectSpace for SequentialObjectSpace {
    fn write<T: Default + Any>(&mut self, obj: T) {
        let default_entry = ObjectSpaceEntry::<T> {
            object_list: Vec::new(),
        };
        let type_id = TypeId::of::<T>();

        let entry = self.typeid_entries_dict
            .entry(type_id)
            .or_insert(Box::new(default_entry));

        if let Some(ent) = entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>() {
            ent.add(obj);
        }
    }

    fn read<T: Default + Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if let Some(entry) = self.typeid_entries_dict.get(&type_id) {
            match entry.as_any_ref().downcast_ref::<ObjectSpaceEntry<T>>() {
                Some(ent) => ent.get(),
                None => None,
            }
        } else {
            None
        }
    }

    fn read_conditional<P, T>(&self, cond: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get(&type_id) {
            Some(entry) => match entry.as_any_ref().downcast_ref::<ObjectSpaceEntry<T>>() {
                Some(ent) => ent.get_conditional(cond),
                None => None,
            },
            _ => None,
        }
    }

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = &T> + 'a>
    where
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get(&type_id) {
            Some(entry) => match entry.as_any_ref().downcast_ref::<ObjectSpaceEntry<T>>() {
                Some(ent) => Box::new(ent.get_all()),
                None => Box::new(iter::empty::<&T>()),
            },
            None => Box::new(iter::empty::<&T>()),
        }
    }

    fn take<T: Default + Any>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get_mut(&type_id) {
            Some(entry) => match entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>() {
                Some(ent) => ent.remove(),
                None => None,
            },
            _ => None,
        }
    }

    fn take_conditional<P, T>(&mut self, cond: P) -> Option<T>
    where
        P: FnMut(&T) -> bool,
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get_mut(&type_id) {
            Some(entry) => match entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>() {
                Some(ent) => ent.remove_conditional(cond),
                None => None,
            },
            _ => None,
        }
    }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get_mut(&type_id) {
            Some(entry) => entry
                .as_any_mut()
                .downcast_mut::<ObjectSpaceEntry<T>>()
                .map_or(Box::new(iter::empty::<T>()), |ent| {
                    Box::new(ent.remove_all())
                }),
            None => Box::new(iter::empty::<T>()),
        }
    }
}

trait ObjectSpaceEntryFamily {
    fn as_any_ref(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

struct ObjectSpaceEntry<T: Default + Any> {
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
    fn add(&mut self, obj: T) {
        &self.object_list.push(obj);
    }

    fn get(&self) -> Option<&T> {
        self.object_list.first()
    }

    fn get_conditional<P>(&self, cond: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
    {
        match self.object_list.iter().position(cond) {
            Some(index) => self.object_list.get(index),
            None => None,
        }
    }

    fn get_all(&self) -> Iter<T> {
        self.object_list.iter()
    }

    fn remove(&mut self) -> Option<T> {
        self.object_list.pop()
    }

    fn remove_conditional<P>(&mut self, cond: P) -> Option<T>
    where
        P: FnMut(&T) -> bool,
    {
        self.object_list
            .iter()
            .position(cond)
            .map(|index| self.object_list.remove(index))
    }

    fn remove_all(&mut self) -> Drain<T> {
        self.object_list.drain(..)
    }
}
