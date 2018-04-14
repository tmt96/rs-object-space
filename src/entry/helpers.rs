use std::borrow::Borrow;
use std::cmp::Ord;
use std::collections::Bound;
use std::collections::{BTreeMap, HashMap};
use std::iter::empty;
use std::ops::RangeBounds;
use std::sync::Arc;

use ordered_float::NotNaN;
use serde_json::map::Map;
use serde_json::value::Value;

use entry::TreeSpaceEntry;

pub fn get_primitive_from_map<U>(map: &BTreeMap<U, Vec<Arc<Value>>>) -> Option<Arc<Value>> {
    for (_, vec) in map.iter() {
        if let Some(result) = vec.get(0) {
            return Some(result.clone());
        }
    }
    None
}

pub fn get_primitive_range<R, U>(
    map: &BTreeMap<U, Vec<Arc<Value>>>,
    condition: R,
) -> Option<Arc<Value>>
where
    R: RangeBounds<U>,
    U: Ord,
{
    for (_, vec) in map.range(condition) {
        if let Some(result) = vec.get(0) {
            return Some(result.clone());
        }
    }
    None
}

pub fn get_primitive_key<U>(map: &BTreeMap<U, Vec<Arc<Value>>>, key: &U) -> Option<Arc<Value>>
where
    U: Ord,
{
    match map.get(key) {
        None => None,
        Some(vec) => vec.get(0).map(|res| res.clone()),
    }
}

pub fn get_all_prims_from_map<'a, U>(
    map: &'a BTreeMap<U, Vec<Arc<Value>>>,
) -> Box<Iterator<Item = Value> + 'a> {
    let iter = map.iter().flat_map(|(_, vec)| {
        vec.iter().map(|item| {
            let val: &Value = item.borrow();
            val.clone()
        })
    });
    Box::new(iter)
}

pub fn get_all_prims_range<'a, R, U>(
    map: &'a BTreeMap<U, Vec<Arc<Value>>>,
    condition: R,
) -> Box<Iterator<Item = Value> + 'a>
where
    R: RangeBounds<U>,
    U: Ord,
{
    let iter = map.range(condition).flat_map(|(_, vec)| {
        vec.iter().filter_map(|item| {
            let val: &Value = item.borrow();
            Some(val.clone())
        })
    });
    Box::new(iter)
}

pub fn get_all_prims_key<'a, U>(
    map: &'a BTreeMap<U, Vec<Arc<Value>>>,
    key: &U,
) -> Box<Iterator<Item = Value> + 'a>
where
    U: Ord,
{
    match map.get(key) {
        None => Box::new(empty()),
        Some(vec) => Box::new(vec.iter().filter_map(|item| {
            let val: &Value = item.borrow();
            Some(val.clone())
        })),
    }
}

pub fn remove_primitive_range<R, U>(
    map: &mut BTreeMap<U, Vec<Arc<Value>>>,
    condition: R,
) -> Option<Arc<Value>>
where
    R: RangeBounds<U>,
    U: Ord,
{
    for (_, vec) in map.range_mut(condition) {
        if let Some(result) = vec.pop() {
            return Some(result.clone());
        }
    }
    None
}

pub fn remove_primitive_key<U>(
    map: &mut BTreeMap<U, Vec<Arc<Value>>>,
    key: &U,
) -> Option<Arc<Value>>
where
    U: Ord,
{
    match map.get_mut(key) {
        None => None,
        Some(vec) => vec.pop().map(|res| res.clone()),
    }
}

pub fn remove_all_prims_range<R, U>(
    map: &mut BTreeMap<U, Vec<Arc<Value>>>,
    condition: R,
) -> Vec<Arc<Value>>
where
    R: RangeBounds<U>,
    U: Ord,
{
    map.range_mut(condition)
        .flat_map(|(_, vec)| vec.drain(..))
        .collect()
}

pub fn remove_all_prims_key<'a, U>(
    map: &'a mut BTreeMap<U, Vec<Arc<Value>>>,
    key: &U,
) -> Vec<Arc<Value>>
where
    U: Ord,
{
    map.get_mut(key)
        .map_or(Vec::new(), |vec| vec.drain(..).collect())
}

pub fn remove_primitive_from_map<U>(map: &mut BTreeMap<U, Vec<Arc<Value>>>) -> Option<Arc<Value>> {
    for (_, vec) in map.iter_mut() {
        if let Some(val) = vec.pop() {
            return Some(val);
        }
    }
    None
}

pub fn remove_object(field_map: &mut HashMap<String, TreeSpaceEntry>) -> Option<Arc<Value>> {
    let result_arc = match field_map.iter().next() {
        Some((_, value)) => value.get_helper(),
        None => None,
    };

    result_arc.map(|arc| {
        remove_value_arc(field_map, &arc);
        arc
    })
}

pub fn remove_value_arc(field_map: &mut HashMap<String, TreeSpaceEntry>, removed_arc: &Arc<Value>) {
    for (k, field) in field_map.iter_mut() {
        let component = (*removed_arc).get(k).unwrap();
        match *field {
            TreeSpaceEntry::BoolLeaf(ref mut lookup_map) => {
                if let Some(vec) = lookup_map.get_mut(&component.as_bool().unwrap()) {
                    vec.retain(|arc| !Arc::ptr_eq(arc, &removed_arc))
                }
            }
            TreeSpaceEntry::IntLeaf(ref mut lookup_map) => {
                if let Some(vec) = lookup_map.get_mut(&component.as_i64().unwrap()) {
                    vec.retain(|arc| !Arc::ptr_eq(arc, &removed_arc))
                }
            }
            TreeSpaceEntry::FloatLeaf(ref mut lookup_map) => {
                if let Some(vec) = lookup_map.get_mut(&NotNaN::from(component.as_f64().unwrap())) {
                    vec.retain(|arc| !Arc::ptr_eq(arc, &removed_arc))
                }
            }
            TreeSpaceEntry::StringLeaf(ref mut lookup_map) => {
                if let Some(vec) = lookup_map.get_mut(component.as_str().unwrap()) {
                    vec.retain(|arc| !Arc::ptr_eq(arc, &removed_arc))
                }
            }
            TreeSpaceEntry::VecLeaf(ref mut vec) => {
                vec.retain(|arc| !Arc::ptr_eq(arc, &removed_arc))
            }
            _ => (),
        }
    }
}

fn convert_float_bound(bound: Bound<&f64>) -> Bound<NotNaN<f64>> {
    match bound {
        Bound::Included(value) => {
            Bound::Included(NotNaN::new(*value).expect("NaN values are not accepted"))
        }
        Bound::Excluded(value) => {
            Bound::Excluded(NotNaN::new(*value).expect("NaN values are not accepted"))
        }
        Bound::Unbounded => Bound::Unbounded,
    }
}

pub fn convert_float_range<R>(range: R) -> (Bound<NotNaN<f64>>, Bound<NotNaN<f64>>)
where
    R: RangeBounds<f64>,
{
    (
        convert_float_bound(range.start()),
        convert_float_bound(range.end()),
    )
}

pub fn flatten(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(flatten_value_map(map)),
        _ => v,
    }
}

pub fn deflatten(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(deflatten_value_map(map)),
        _ => v,
    }
}

fn flatten_value_map(map: Map<String, Value>) -> Map<String, Value> {
    let mut result = Map::new();
    for (key, value) in map {
        match value {
            Value::Object(obj) => for (k, v) in obj {
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
    for (key, value) in map {
        let mut iterator = key.splitn(2, '.');
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

    for (key, value) in temp {
        result.insert(key, deflatten(value));
    }
    result
}
