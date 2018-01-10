use std::collections::HashMap;
use std::any::{Any, TypeId};

pub trait ObjectSpace {
    fn write<T: Default + Any + 'static>(&mut self, obj: T);
    fn read<T: Default + Any + 'static>(&self) -> Option<&T>;
    fn read_conditional<T: Default + Any + 'static>(&self, cond: fn(T) -> bool) -> Option<&T>;
    fn take<T: Default + Any + 'static>(&mut self) -> Option<T>;
    fn take_conditional<T: Default + Any + 'static>(&self, cond: fn(T) -> bool) -> Option<T>;
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
    fn write<T: Default + Any + 'static>(&mut self, obj: T) {
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

    fn read<T: Default + Any + 'static>(&self) -> Option<&T> {
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

    fn read_conditional<T: Default + Any + 'static>(&self, cond: fn(T) -> bool) -> Option<&T> {
        None
    }

    fn take<T: Default + Any + 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        if let Some(entry) = self.typeid_entries_dict.get_mut(&type_id) {
            match entry.as_any_mut().downcast_mut::<ObjectSpaceEntry<T>>() {
                Some(ent) => ent.remove(),
                None => None,
            }
        } else {
            None
        }
    }

    fn take_conditional<T: Default + Any + 'static>(&self, cond: fn(T) -> bool) -> Option<T> {
        None
    }
}

trait ObjectSpaceEntryFamily {
    fn as_any_ref(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

struct ObjectSpaceEntry<T: Default + Any + 'static> {
    object_list: Vec<T>,
}

impl<T> ObjectSpaceEntryFamily for ObjectSpaceEntry<T>
where
    T: Default + Any + 'static,
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
    T: Default + Any + 'static,
{
    fn add(&mut self, obj: T) {
        &self.object_list.push(obj);
    }

    fn get(&self) -> Option<&T> {
        self.object_list.first()
    }

    fn remove(&mut self) -> Option<T> {
        self.object_list.pop()
    }
}
