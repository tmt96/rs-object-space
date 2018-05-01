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
        Value::Object(map) => Value::Object(deflatten_value_map(map)),
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
