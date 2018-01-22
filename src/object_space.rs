use std::collections::HashMap;
use std::any::TypeId;
use std::iter;
use entry::TreeSpaceEntry;
use serde::{Deserialize, Serialize};

pub trait ObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn try_read_conditional<P, T>(&self, cond: &P) -> Option<&T>
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>;

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn read_all_conditional<'a, P, T>(&'a self, cond: P) -> Box<Iterator<Item = &T> + 'a>
    // where
    //     for<'r> P: Fn(&'r &T) -> bool + 'a,
    //     for <'de> T: Serialize + Deserialize<'de>;

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn read_conditional<P, T>(&self, cond: &P) -> &T
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>;

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn try_take_conditional<P, T>(&mut self, cond: &P) -> Option<T>
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>;

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn take_all_conditional<'a, P, T>(&'a mut self, cond: P) -> Box<Iterator<Item = T> + 'a>
    // where
    //     for<'r> P: Fn(&'r mut T) -> bool + 'a,
    //     for <'de> T: Serialize + Deserialize<'de>;

    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    // fn take_conditional<P, T>(&mut self, cond: &P) -> T
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>;
}

pub struct SequentialObjectSpace {
    typeid_entries_dict: HashMap<TypeId, TreeSpaceEntry>,
}

impl SequentialObjectSpace {
    pub fn new() -> SequentialObjectSpace {
        SequentialObjectSpace {
            typeid_entries_dict: HashMap::new(),
        }
    }

    fn get_object_entry_ref<T>(&self) -> Option<&TreeSpaceEntry>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get(&type_id)
    }

    fn get_object_entry_mut<T>(&mut self) -> Option<&mut TreeSpaceEntry>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.typeid_entries_dict.get_mut(&type_id)
    }
}

impl ObjectSpace for SequentialObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        let default_entry = TreeSpaceEntry::new();
        let type_id = TypeId::of::<T>();

        self.typeid_entries_dict
            .entry(type_id)
            .or_insert(default_entry)
            .add(obj);
    }

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get::<T>(),
            _ => None,
        }
    }

    // fn try_read_conditional<P, T>(&self, cond: &P) -> Option<&T>
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     match self.get_object_entry_ref::<T>() {
    //         Some(entry) => entry.get_conditional(cond),
    //         _ => None,
    //     }
    // }

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all::<T>()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    // fn read_all_conditional<'a, P, T>(&'a self, cond: P) -> Box<Iterator<Item = &T> + 'a>
    // where
    //     for<'r> P: Fn(&'r &T) -> bool + 'a,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     match self.get_object_entry_ref::<T>() {
    //         Some(ent) => Box::new(ent.get_all_conditional(cond).into_iter()),
    //         None => Box::new(iter::empty::<&T>()),
    //     }
    // }

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        loop {
            if let Some(item) = self.try_read::<T>() {
                return item;
            }
        }
    }

    // fn read_conditional<P, T>(&self, cond: &P) -> &T
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     loop {
    //         if let Some(item) = self.try_read_conditional::<P, T>(cond) {
    //             return item;
    //         }
    //     }
    // }

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => ent.remove::<T>(),
            None => None,
        }
    }

    // fn try_take_conditional<P, T>(&mut self, cond: &P) -> Option<T>
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     match self.get_object_entry_mut::<T>() {
    //         Some(ent) => ent.remove_conditional(cond),
    //         None => None,
    //     }
    // }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => Box::new(entry.remove_all::<T>().into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

    // fn take_all_conditional<'a, P, T>(&'a mut self, cond: P) -> Box<Iterator<Item = T> + 'a>
    // where
    //     for<'r> P: Fn(&'r mut T) -> bool + 'a,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     match self.get_object_entry_mut::<T>() {
    //         Some(entry) => Box::new(entry.remove_all_conditional(cond).into_iter()),
    //         None => Box::new(iter::empty::<T>()),
    //     }
    // }

    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        loop {
            if let Some(item) = self.try_take::<T>() {
                return item;
            }
        }
    }

    // fn take_conditional<P, T>(&mut self, cond: &P) -> T
    // where
    //     P: Fn(&T) -> bool,
    //     for <'de> T: Serialize + Deserialize<'de>,
    // {
    //     loop {
    //         if let Some(item) = self.try_take_conditional::<P, T>(cond) {
    //             return item;
    //         }
    //     }
    // }
}
