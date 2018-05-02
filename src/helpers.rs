use std::iter::Peekable;

use serde_json::map::Map;
use serde_json::value::Value;

pub fn flatten(v: Value) -> Value {
    match v {
        Value::Object(_) => {
            let mut result = Map::new();
            flatten_helper("", v, &mut result);
            Value::Object(result)
        }
        _ => v,
    }
}

pub fn deflatten(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(deflatten_helper(map)),
        _ => v,
    }
}

fn flatten_helper(path: &str, value: Value, result: &mut Map<String, Value>) {
    match value {
        Value::Object(map) => for (k, v) in map {
            let new_path = if path.is_empty() {
                k
            } else {
                format!("{}.{}", path, k)
            };
            flatten_helper(&new_path, v, result)
        },
        _ => {
            result.insert(path.to_owned(), value);
        }
    }
}

fn deflatten_helper(map: Map<String, Value>) -> Map<String, Value> {
    let mut result = Map::new();
    for (key, value) in map {
        insert_to_map(&mut result, &mut key.split('.').peekable(), value);
    }
    result
}

fn insert_to_map<'a, I>(map: &mut Map<String, Value>, iter: &mut Peekable<I>, value: Value)
where
    I: Iterator<Item = &'a str>,
{
    if let Some(field_val) = iter.next() {
        match iter.peek() {
            Some(_) => {
                let child = map.entry(field_val).or_insert(Value::Object(Map::new()));
                insert_to_map(child.as_object_mut().expect("invalid JSON"), iter, value);
            }
            None => {
                map.insert(field_val.to_owned(), value);
            }
        }
    }
}
