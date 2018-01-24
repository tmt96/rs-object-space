use std::collections::HashMap;
use std::any::TypeId;
use std::iter;
use std::collections::range::RangeArgument;
use serde::{Deserialize, Serialize};

use entry::TreeSpaceEntry;
use entry::ConditionalEntry;

pub trait ObjectSpaceConditional<U>: ObjectSpace {
    fn try_read_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn read_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn read_conditional<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn try_take_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn take_all_conditional<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;

    fn take_conditional<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<U> + Clone;
}

pub trait ObjectSpace {
    fn write<T>(&mut self, obj: T)
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn try_read<T>(&self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn read<T>(&self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;

    fn take<T>(&mut self) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static;
}

pub struct TreeObjectSpace {
    typeid_entries_dict: HashMap<TypeId, TreeSpaceEntry>,
}

impl TreeObjectSpace {
    pub fn new() -> TreeObjectSpace {
        TreeObjectSpace {
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

impl ObjectSpace for TreeObjectSpace {
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

    fn read_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all::<T>()),
            None => Box::new(iter::empty::<T>()),
        }
    }

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

    fn try_take<T>(&mut self) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => ent.remove::<T>(),
            None => None,
        }
    }

    fn take_all<'a, T>(&'a mut self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => Box::new(entry.remove_all::<T>().into_iter()),
            None => Box::new(iter::empty::<T>()),
        }
    }

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
}

impl ObjectSpaceConditional<i64> for TreeObjectSpace {
    fn try_read_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_conditional::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_conditional<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_conditional<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(
                ent.remove_all_conditional::<T, _>(field, condition)
                    .into_iter(),
            ),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_conditional<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<i64> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}

impl ObjectSpaceConditional<String> for TreeObjectSpace {
    fn try_read_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_conditional::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_conditional<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_conditional<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(
                ent.remove_all_conditional::<T, _>(field, condition)
                    .into_iter(),
            ),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_conditional<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<String> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}

impl ObjectSpaceConditional<bool> for TreeObjectSpace {
    fn try_read_conditional<T, R>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(entry) => entry.get_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn read_all_conditional<'a, T, R>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_ref::<T>() {
            Some(ent) => Box::new(ent.get_all_conditional::<T, _>(field, condition)),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn read_conditional<T, R>(&self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }

    fn try_take_conditional<T, R>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(entry) => entry.remove_conditional::<T, _>(field, condition),
            _ => None,
        }
    }

    fn take_all_conditional<'a, T, R>(
        &'a mut self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        match self.get_object_entry_mut::<T>() {
            Some(ent) => Box::new(
                ent.remove_all_conditional::<T, _>(field, condition)
                    .into_iter(),
            ),
            None => Box::new(iter::empty::<T>()),
        }
    }

    fn take_conditional<T, R>(&mut self, field: &str, condition: R) -> T
    where
        for<'de> T: Serialize + Deserialize<'de> + 'static,
        R: RangeArgument<bool> + Clone,
    {
        loop {
            if let Some(item) = self.try_read_conditional::<T, _>(field, condition.clone()) {
                return item;
            }
        }
    }
}
