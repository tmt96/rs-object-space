use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::iter;
use entry::{ObjectSpaceEntry, ObjectSpaceEntryFamily};

pub trait ObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        T: Default + Any;

    fn try_read<T>(&self) -> Option<&T>
    where
        T: Default + Any;

    fn try_read_conditional<P, T>(&self, cond: &P) -> Option<&T>
    where
        P: Fn(&T) -> bool,
        T: Default + Any;

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = &T> + 'a>
    where
        T: Default + Any;

    fn read_all_conditional<'a, P, T>(&'a self, cond: P) -> Box<Iterator<Item = &T> + 'a>
    where
        for<'r> P: Fn(&'r &T) -> bool + 'a,
        T: Default + Any;

    fn read<T>(&self) -> &T
    where
        T: Default + Any;

    fn read_conditional<P, T>(&self, cond: &P) -> &T
    where
        P: Fn(&T) -> bool,
        T: Default + Any;

    fn try_take<T>(&mut self) -> Option<T>
    where
        T: Default + Any;

    fn try_take_conditional<P, T>(&mut self, cond: &P) -> Option<T>
    where
        P: Fn(&T) -> bool,
        T: Default + Any;

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        T: Default + Any;

    fn take_all_conditional<'a, P, T>(&'a mut self, cond: P) -> Box<Iterator<Item = T> + 'a>
    where
        for<'r> P: Fn(&'r mut T) -> bool + 'a,
        T: Default + Any;

    fn take<T>(&mut self) -> T
    where
        T: Default + Any;

    fn take_conditional<P, T>(&mut self, cond: &P) -> T
    where
        P: Fn(&T) -> bool,
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

    fn get_object_entry_ref<T>(&self) -> Option<&ObjectSpaceEntry<T>>
    where
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get(&type_id) {
            Some(entry) => entry.as_any_ref().downcast_ref::<ObjectSpaceEntry<T>>(),
            _ => None,
        }
    }

    fn get_object_entry_mut<T>(&mut self) -> Option<&mut ObjectSpaceEntry<T>>
    where
        T: Default + Any,
    {
        let type_id = TypeId::of::<T>();

        match self.typeid_entries_dict.get_mut(&type_id) {
            Some(entry) => entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>(),
            _ => None,
        }
    }
}

impl ObjectSpace for SequentialObjectSpace {
    fn write<T: Default + Any>(&mut self, obj: T) {
        let default_entry = ObjectSpaceEntry::<T>::new();
        let type_id = TypeId::of::<T>();

        let entry = self.typeid_entries_dict
            .entry(type_id)
            .or_insert(Box::new(default_entry));

        if let Some(ent) = entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>() {
            ent.add(obj);
        }
    }

    fn try_read<T: Default + Any>(&self) -> Option<&T> {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get(),
            _ => None,
        }
    }

    fn try_read_conditional<P, T>(&self, cond: &P) -> Option<&T>
    where
        P: Fn(&T) -> bool,
        T: Default + Any,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_conditional(cond),
            _ => None,
        }
    }

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = &T> + 'a>
    where
        T: Default + Any,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all()),
            None => Box::new(iter::empty::<&T>()),
        }
    }

    fn read_all_conditional<'a, P, T>(&'a self, cond: P) -> Box<Iterator<Item = &T> + 'a>
    where
        for<'r> P: Fn(&'r &T) -> bool + 'a,
        T: Default + Any,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_conditional(cond)),
            None => Box::new(iter::empty::<&T>()),
        }
    }

    fn read<T>(&self) -> &T
    where
        T: Default + Any,
    {
        loop {
            if let Some(item) = self.try_read::<T>() {
                return item;
            }
        }
    }

    fn read_conditional<P, T>(&self, cond: &P) -> &T
    where
        P: Fn(&T) -> bool,
        T: Default + Any,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<P, T>(cond) {
                return item;
            }
        }
    }

    fn try_take<T: Default + Any>(&mut self) -> Option<T> {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => ent.remove(),
            None => None,
        }
    }

    fn try_take_conditional<P, T>(&mut self, cond: &P) -> Option<T>
    where
        P: Fn(&T) -> bool,
        T: Default + Any,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => ent.remove_conditional(cond),
            None => None,
        }
    }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        T: Default + Any,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => Box::new(entry.remove_all()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_all_conditional<'a, P, T>(&'a mut self, cond: P) -> Box<Iterator<Item = T> + 'a>
    where
        for<'r> P: Fn(&'r mut T) -> bool + 'a,
        T: Default + Any,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => Box::new(entry.remove_all_conditional(cond)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take<T>(&mut self) -> T
    where
        T: Default + Any,
    {
        loop {
            if let Some(item) = self.try_take::<T>() {
                return item;
            }
        }
    }

    fn take_conditional<P, T>(&mut self, cond: &P) -> T
    where
        P: Fn(&T) -> bool,
        T: Default + Any,
    {
        loop {
            if let Some(item) = self.try_take_conditional::<P, T>(cond) {
                return item;
            }
        }
    }
}
