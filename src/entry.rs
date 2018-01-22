use std::any::Any;
use std::clone::Clone;
use std::iter::{empty, FromIterator};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Borrow;
use std::fmt::Debug;

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
            TreeSpaceEntry::IntLeaf(ref int_map) => get_primitive_from_map(int_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_primitive_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => {
                if let Some(result) = vec.get(0) {
                    let val: &Value = result.borrow();
                    return from_value(deflatten(val.clone())).ok();
                }
                None
            }
            TreeSpaceEntry::Branch(ref object_field_map) => {
                if let Some((_, value)) = object_field_map.iter().next() {
                    return value.get::<T>();
                }
                None
            }
        }
    }

    pub fn get_all<'a, T>(&'a self) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'a,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::BoolLeaf(ref bool_map) => get_all_prims_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_all_prims_from_map(int_map),
            TreeSpaceEntry::StringLeaf(ref string_map) => get_all_prims_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref vec) => Box::new(vec.iter().filter_map(|item| {
                let val: &Value = item.borrow();
                from_value(deflatten(val.clone())).ok()
            })),
            TreeSpaceEntry::Branch(ref object_field_map) => {
                if let Some((_, value)) = object_field_map.iter().next() {
                    return value.get_all::<T>();
                }
                Box::new(empty())
            }
        }
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        match self.remove_helper() {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn remove_all<T>(&mut self) -> Vec<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        Vec::new()
    }

    fn remove_helper(&mut self) -> Option<Arc<Value>> {
        match *self {
            TreeSpaceEntry::Null => None,
            TreeSpaceEntry::BoolLeaf(ref mut bool_map) => remove_primitive_from_map(bool_map),
            TreeSpaceEntry::IntLeaf(ref mut int_map) => remove_primitive_from_map(int_map),
            TreeSpaceEntry::StringLeaf(ref mut string_map) => remove_primitive_from_map(string_map),
            TreeSpaceEntry::VecLeaf(ref mut vec) => vec.pop(),
            TreeSpaceEntry::Branch(ref mut object_field_map) => remove_object(object_field_map),
        }
    }

    fn add_value_by_num(&mut self, num: Number, value: Arc<Value>) {
        if let Some(i) = num.as_i64() {
            self.add_value_by_int(i, value);
        } else if let Some(f) = num.as_f64() {
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
    let mut iter = map.iter();
    while let Some((_, vec)) = iter.next() {
        if let Some(result) = vec.get(0) {
            let val: &Value = result.borrow();
            return from_value(deflatten(val.clone())).ok();
        }
    }
    None
}

fn get_all_prims_from_map<'a, T, U>(
    map: &'a BTreeMap<U, Vec<Arc<Value>>>,
) -> Box<Iterator<Item = T> + 'a>
where
    for<'de> T: Deserialize<'de>,
{
    let iter = map.iter().flat_map(|(_, vec)| {
        vec.iter().filter_map(|item| {
            let val: &Value = item.borrow();
            from_value(deflatten(val.clone())).ok()
        })
    });
    Box::new(iter)
}

fn remove_primitive_from_map<U>(map: &mut BTreeMap<U, Vec<Arc<Value>>>) -> Option<Arc<Value>> {
    let mut iter = map.iter_mut();
    while let Some((_, vec)) = iter.next() {
        println!("hello");
        if let Some(val) = vec.pop() {
            return Some(val);
        }
    }
    None
}

fn remove_object(field_map: &mut HashMap<String, TreeSpaceEntry>) -> Option<Arc<Value>> {
    let mut iter = field_map.iter_mut();
    if let Some((_, value)) = iter.next() {
        if let Some(result_arc) = value.remove_helper() {
            for (k, field) in iter {
                let component = (*result_arc).get(k).unwrap();
                match *field {
                    TreeSpaceEntry::BoolLeaf(ref mut lookup_map) => {
                        match lookup_map.get_mut(&component.as_bool().unwrap()) {
                            Some(vec) => vec.retain(|arc| !Arc::ptr_eq(arc, &result_arc)),
                            None => (),
                        }
                    }
                    TreeSpaceEntry::IntLeaf(ref mut lookup_map) => {
                        match lookup_map.get_mut(&component.as_i64().unwrap()) {
                            Some(vec) => vec.retain(|arc| !Arc::ptr_eq(arc, &result_arc)),
                            None => (),
                        }
                    }
                    TreeSpaceEntry::StringLeaf(ref mut lookup_map) => {
                        match lookup_map.get_mut(component.as_str().unwrap()) {
                            Some(vec) => vec.retain(|arc| !Arc::ptr_eq(arc, &result_arc)),
                            None => (),
                        }
                    }
                    TreeSpaceEntry::VecLeaf(ref mut vec) => {
                        vec.retain(|arc| !Arc::ptr_eq(arc, &result_arc))
                    }
                    _ => (),
                }
            }
            return Some(result_arc);
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

    #[macro_use]
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct TestStruct {
        count: i32,
        name: String,
        eaten: bool,
    }

    #[test]
    fn get() {
        let mut string_entry = TreeSpaceEntry::new();
        assert_eq!(string_entry.get::<String>(), None);
        string_entry.add(String::from("Hello World"));
        assert_eq!(
            string_entry.get::<String>(),
            Some(String::from("Hello World"))
        );
        assert_ne!(string_entry.get::<String>(), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        assert_eq!(
            test_struct_entry.get::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
    }

    #[test]
    fn remove() {
        let mut entry = TreeSpaceEntry::new();
        assert_eq!(entry.remove::<String>(), None);
        entry.add(String::from("Hello World"));
        assert_eq!(entry.remove::<String>(), Some(String::from("Hello World")));
        assert_eq!(entry.remove::<String>(), None);

        let mut test_struct_entry = TreeSpaceEntry::new();
        test_struct_entry.add(TestStruct {
            count: 3,
            name: String::from("Tuan"),
        });
        assert_eq!(
            test_struct_entry.remove::<TestStruct>(),
            Some(TestStruct {
                count: 3,
                name: String::from("Tuan"),
            })
        );
        assert_eq!(test_struct_entry.remove::<TestStruct>(), None);
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
