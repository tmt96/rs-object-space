use std::any::Any;
use std::clone::Clone;
use std::iter::FromIterator;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Borrow;

use serde_json::value::{from_value, to_value, Value};
use serde_json::map::Map;
use serde_json::Number;
use serde::ser::Serialize;
use serde::de::Deserialize;

pub trait ObjectSpaceEntryFamily {
    fn as_any_ref(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

pub struct ObjectSpaceEntry<T: Clone + Any> {
    object_list: Vec<T>,
}

impl<T> ObjectSpaceEntryFamily for ObjectSpaceEntry<T>
where
    T: Clone + Any,
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
    T: Clone + Any,
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

    // pub fn get_conditional<P>(&self, cond: &P) -> Option<&T>
    // where
    //     P: Fn(&T) -> bool,
    // {
    //     match self.object_list.iter().position(cond) {
    //         Some(index) => self.object_list.get(index),
    //         None => None,
    //     }
    // }

    pub fn get_all(&self) -> Vec<&T> {
        Vec::from_iter(self.object_list.iter())
    }

    // pub fn get_all_conditional<P>(&self, cond: P) -> Vec<&T>
    // where
    //     for<'r> P: Fn(&'r &T) -> bool,
    // {
    //     Vec::from_iter(self.object_list.iter().filter(cond))
    // }

    pub fn remove(&mut self) -> Option<T> {
        self.object_list.pop()
    }

    // pub fn remove_conditional<'a, P>(&mut self, cond: &P) -> Option<T>
    // where
    //     P: Fn(&T) -> bool,
    // {
    //     self.object_list
    //         .iter()
    //         .position(cond)
    //         .map(|index| self.object_list.remove(index))
    // }

    pub fn remove_all(&mut self) -> Vec<T> {
        let result = self.object_list.clone();
        self.object_list = Vec::new();
        result
    }

    // pub fn remove_all_conditional<P>(&mut self, cond: P) -> Vec<T>
    // where
    //     for<'r> P: Fn(&'r mut T) -> bool,
    // {
    //     Vec::from_iter(self.object_list.drain_filter(cond))
    // }

    fn len(&self) -> usize {
        self.object_list.len()
    }
}

enum TreeSpaceEntry {
    IntLeaf(BTreeMap<i64, Vec<Arc<Value>>>),
    BoolLeaf(BTreeMap<bool, Vec<Arc<Value>>>),
    StringLeaf(BTreeMap<String, Vec<Arc<Value>>>),
    VecLeaf(Vec<Arc<Value>>),
    Branch(HashMap<String, TreeSpaceEntry>),
    Null,
}

impl TreeSpaceEntry {
    pub fn as_any_ref(&self) -> &Any {
        self
    }

    pub fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    pub fn new() -> Self {
        TreeSpaceEntry::Null
    }

    pub fn add<T>(&mut self, obj: T)
    where
        T: Serialize,
    {
        match to_value(obj) {
            Ok(value) => {
                let flattened_val = flatten(value);
                match flattened_val.clone() {
                    Value::Number(num) => self.add_value_by_num(num, Arc::new(flattened_val)),
                    Value::Bool(boolean) => {
                        self.add_value_by_bool(boolean, Arc::new(flattened_val))
                    }
                    Value::String(string) => {
                        self.add_value_by_string(string, Arc::new(flattened_val))
                    }
                    Value::Array(vec) => self.add_value_by_array(vec, Arc::new(flattened_val)),
                    Value::Object(map) => self.add_value_by_object(map, Arc::new(flattened_val)),
                    _ => (),
                }
            }
            Err(e) => panic!("struct not serializable: {:?}", e),
        }
    }

    pub fn get<T>(&self) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref bool_map) => get_primitive_from_map(bool_map),
            TreeSpaceEntry::Branch(ref object_field_map) => None,
            TreeSpaceEntry::IntLeaf(ref int_map) => get_primitive_from_map(int_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_primitive_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => None,
        }
    }

    pub fn get_all<'de, T>(&self) -> Vec<&T>
    where
        T: Deserialize<'de>,
    {
        Vec::new()
    }

    pub fn remove<'de, T>(&mut self) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        None
    }

    pub fn remove_all<'de, T>(&mut self) -> Vec<T>
    where
        T: Deserialize<'de>,
    {
        Vec::new()
    }

    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        if let Some(i) = value.as_i64() {
            self.add_value_by_int(i, value);
        } else if let Some(f) = value.as_f64() {
            self.add_value_by_float(f, value);
        } else {
            panic!("Not a number!");
        }
    }

    fn add_value_by_int(&mut self, i: i64, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::IntLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::IntLeaf(ref mut map) => {
                let vec = map.entry(i).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found int."),
        }
    }

    fn add_value_by_float(&mut self, f: f64, value: Arc<Value>) {}

    fn add_value_by_string(&mut self, string: String, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::StringLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::StringLeaf(ref mut map) => {
                let vec = map.entry(string).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found String."),
        }
    }

    fn add_value_by_bool(&mut self, boolean: bool, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::BoolLeaf(BTreeMap::new());
        }

        match *self {
            TreeSpaceEntry::BoolLeaf(ref mut map) => {
                let vec = map.entry(boolean).or_insert(Vec::new());
                vec.push(value);
            }
            _ => panic!("Incorrect data type! Found bool."),
        }
    }

    fn add_value_by_array(&mut self, vec: Vec<Value>, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::VecLeaf(Vec::new());
        }

        match *self {
            TreeSpaceEntry::VecLeaf(ref mut vec) => vec.push(value),
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    fn add_value_by_object(&mut self, map: Map<String, Value>, value: Arc<Value>) {
        if let &mut TreeSpaceEntry::Null = self {
            *self = TreeSpaceEntry::Branch(HashMap::new());
        }

        match *self {
            TreeSpaceEntry::Branch(ref mut hashmap) => for (key, val) in map.into_iter() {
                let sub_entry = hashmap.entry(key).or_insert(TreeSpaceEntry::Null);
                match val.clone() {
                    Value::Number(num) => sub_entry.add_value_by_num(num, value.clone()),
                    Value::Bool(boolean) => sub_entry.add_value_by_bool(boolean, value.clone()),
                    Value::String(string) => sub_entry.add_value_by_string(string, value.clone()),
                    Value::Array(vec) => sub_entry.add_value_by_array(vec, value.clone()),
                    Value::Object(map) => panic!("Incorrect data type! Found object."),
                    _ => (),
                }
            },
            _ => panic!("Incorrect data type! Found object."),
        }
    }
}

fn get_primitive_from_map<T, U>(map: &BTreeMap<U, Vec<Arc<Value>>>) -> Option<T>
where
    for<'de> T: Deserialize<'de>,
{
    while let Some((_, vec)) = map.iter().next() {
        if let Some(result) = vec.get(0) {
            let val: &Value = result.borrow();
            return from_value(deflatten(val.clone())).ok();
        }
    }
    None
}

fn flatten(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(flatten_value_map(map)),
        _ => v,
    }
}

fn deflatten(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(deflatten_value_map(map)),
        _ => v,
    }
}

fn flatten_value_map(map: Map<String, Value>) -> Map<String, Value> {
    let mut result = Map::new();
    for (key, value) in map.into_iter() {
        match value {
            Value::Object(obj) => for (k, v) in obj.into_iter() {
                let new_key = format!("{}.{}", key, k);
                result.insert(new_key, flatten(v));
            },
            _ => {
                result.insert(key, value);
                ()
            }
        };
    }
    result
}

fn deflatten_value_map(map: Map<String, Value>) -> Map<String, Value> {
    let mut temp = Map::new();
    let mut result = Map::new();
    for (key, value) in map.into_iter() {
        let mut iterator = key.splitn(2, ".");
        if let Some(newkey) = iterator.next() {
            match iterator.next() {
                None => {
                    result.entry(newkey).or_insert(value);
                    ()
                }
                Some(subkey) => {
                    let submap = temp.entry(newkey).or_insert(Value::Object(Map::new()));
                    if let Some(submap) = submap.as_object_mut() {
                        submap.entry(subkey).or_insert(value);
                    }
                }
            }
        }
    }

    for (key, value) in temp.into_iter() {
        result.insert(key, deflatten(value));
    }
    result
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
        assert_eq!(entry.get_all().len(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        assert_eq!(entry.get_all(), vec!["Hello", "World"]);
        assert_ne!(entry.len(), 0);
    }

    #[test]
    fn remove_all() {
        let mut entry = ObjectSpaceEntry::<String>::new();
        assert_eq!(entry.remove_all().len(), 0);
        entry.add("Hello".to_string());
        entry.add("World".to_string());
        assert_eq!(entry.remove_all(), vec!["Hello", "World"]);
        assert_eq!(entry.len(), 0);
    }
}
