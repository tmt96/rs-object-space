use std::collections::Bound;
use std::collections::{BTreeMap, HashMap};
use std::iter::empty;
use std::ops::RangeBounds;

use indexmap::IndexSet;
use ordered_float::NotNaN;
use serde_json::map::Map;
use serde_json::value::Value;
use serde_json::Number;

pub enum ValueIndexer {
    FloatLeaf(BTreeMap<NotNaN<f64>, IndexSet<u64>>),
    IntLeaf(BTreeMap<i64, IndexSet<u64>>),
    BoolLeaf(BTreeMap<bool, IndexSet<u64>>),
    StringLeaf(BTreeMap<String, IndexSet<u64>>),
    VecLeaf(IndexSet<u64>),
    Branch(HashMap<String, ValueIndexer>),
    Null,
}

impl Default for ValueIndexer {
    fn default() -> Self {
        ValueIndexer::Null
    }
}

impl ValueIndexer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, obj: Value, index: u64) {
        match obj {
            Value::Number(num) => self.add_value_by_num(num, index),
            Value::Bool(boolean) => self.add_index(boolean, index),
            Value::String(string) => self.add_index(string, index),
            Value::Array(_) => self.add_value_by_array(index),
            Value::Object(map) => self.add_value_by_object(map, index),
            _ => (),
        }
    }

    pub fn remove(&mut self, index: u64, value: &Value) {
        match value {
            Value::Number(num) => self.remove_by_num(num, index),
            Value::Bool(boolean) => self.remove_index(boolean, index),
            Value::String(string) => self.remove_index(string, index),
            Value::Array(_) => self.remove_by_array(index),
            Value::Object(map) => self.remove_by_object(map, index),
            _ => (),
        }
    }

    fn add_value_by_num(&mut self, num: Number, index: u64) {
        // only parse as f64 if it is actually f64
        // (e.g: accept '64.0' but not '64')
        if num.is_f64() {
            self.add_index(num.as_f64().unwrap(), index);
        } else if let Some(i) = num.as_i64() {
            self.add_index(i, index);
        } else {
            panic!("Not a number!");
        }
    }

    fn remove_by_num(&mut self, num: &Number, index: u64) {
        // only parse as f64 if it is actually f64
        // (e.g: accept '64.0' but not '64')
        if num.is_f64() {
            self.remove_index(&num.as_f64().unwrap(), index);
        } else if let Some(i) = num.as_i64() {
            self.remove_index(&i, index);
        } else {
            panic!("Not a number!");
        }
    }

    fn add_value_by_array(&mut self, index: u64) {
        if let ValueIndexer::Null = *self {
            *self = ValueIndexer::VecLeaf(IndexSet::new());
        }
        match *self {
            ValueIndexer::VecLeaf(ref mut set) => {
                set.insert(index);
            }
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    fn remove_by_array(&mut self, index: u64) {
        match *self {
            ValueIndexer::VecLeaf(ref mut set) => {
                set.remove(&index);
            }
            _ => panic!("Incorrect data type! Found vec."),
        }
    }

    fn add_value_by_object(&mut self, map: Map<String, Value>, index: u64) {
        if let ValueIndexer::Null = *self {
            *self = ValueIndexer::Branch(HashMap::new());
        }

        match *self {
            ValueIndexer::Branch(ref mut hashmap) => for (key, val) in map {
                let sub_entry = hashmap.entry(key).or_insert(ValueIndexer::Null);
                sub_entry.add(val, index);
            },
            _ => panic!("Incorrect data type! Found object."),
        }
    }

    fn remove_by_object(&mut self, map: &Map<String, Value>, index: u64) {
        if let ValueIndexer::Null = *self {
            *self = ValueIndexer::Branch(HashMap::new());
        }

        match *self {
            ValueIndexer::Branch(ref mut hashmap) => for (key, val) in map {
                hashmap
                    .get_mut(key)
                    .map(|indexer| indexer.remove(index, val));
            },
            _ => panic!("Incorrect data type! Found object."),
        }
    }
}

trait Indexer<T> {
    fn add_index(&mut self, field_value: T, index: u64);

    fn remove_index(&mut self, field_value: &T, index: u64);
}

macro_rules! impl_indexer {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl Indexer<$ty> for ValueIndexer {
                fn add_index(&mut self, field_value: $ty, index: u64) {
                    if let ValueIndexer::Null = *self {
                        *self = ValueIndexer::$path(BTreeMap::new());
                    }

                    match *self {
                        ValueIndexer::$path(ref mut map) => {
                            let set = map.entry(field_value).or_insert(IndexSet::new());
                            set.insert(index);
                        }
                        _ => panic!("Incorrect data type!"),
                    }
                }

                fn remove_index(&mut self, field_value: &$ty, index: u64) {
                    match *self {
                        ValueIndexer::$path(ref mut map) => {
                            map.get_mut(field_value).map(|set| set.remove(&index));
                        }
                        _ => panic!("Incorrect data type!"),
                    }
                }
            }
        )*
    };
}

impl_indexer!{[IntLeaf, i64] [StringLeaf, String] [BoolLeaf, bool] [FloatLeaf, NotNaN<f64>] }

impl Indexer<f64> for ValueIndexer {
    fn add_index(&mut self, field_value: f64, index: u64) {
        self.add_index(
            NotNaN::new(field_value).expect("cannot convert an NaN value"),
            index,
        )
    }

    fn remove_index(&mut self, field_value: &f64, index: u64) {
        self.remove_index(
            &NotNaN::new(*field_value).expect("cannot convert an NaN value"),
            index,
        )
    }
}

pub trait ValueLookupIndexer<T> {
    fn get_index_by_value(&self, field: &str, key: &T) -> Option<u64>;

    fn get_all_indices_by_value<'a>(
        &'a self,
        field: &str,
        key: &T,
    ) -> Box<Iterator<Item = u64> + 'a>;
}

macro_rules! impl_value_lookup_indexer {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl ValueLookupIndexer<$ty> for ValueIndexer {
                fn get_index_by_value(&self, field: &str, key: &$ty) -> Option<u64> {
                    match *self {
                        ValueIndexer::Null => None,
                        ValueIndexer::$path(ref map) => map.get(key).and_then(|set| set.get_index(0).map(|i| *i)),
                        ValueIndexer::Branch(ref field_map) => field_map
                            .get(field)
                            .and_then(|entry| entry.get_index_by_value("", key)),
                        _ => panic!("Not correct type"),
                    }
                }

                fn get_all_indices_by_value<'a>(&'a self, field: &str, key: &$ty)
                    -> Box<Iterator<Item = u64> + 'a> {
                    match *self {
                        ValueIndexer::Null => Box::new(empty()),
                        ValueIndexer::$path(ref map) => map
                            .get(key)
                            .map_or(
                                Box::new(empty()), |set| Box::new(set.iter().cloned())
                            ),
                        ValueIndexer::Branch(ref field_map) => field_map
                            .get(field)
                            .map_or(
                                Box::new(empty()),
                                |entry| entry.get_all_indices_by_value("", key)
                            ),
                        _ => panic!("Not correct type"),
                    }
                }
            }
        )*   
    };
}

impl_value_lookup_indexer!{ [IntLeaf, i64] [StringLeaf, String] [BoolLeaf, bool] [FloatLeaf, NotNaN<f64>] }

impl ValueLookupIndexer<f64> for ValueIndexer {
    fn get_index_by_value(&self, field: &str, key: &f64) -> Option<u64> {
        self.get_index_by_value(
            field,
            &NotNaN::new(*key).expect("NaN value is not accepted"),
        )
    }

    fn get_all_indices_by_value<'a>(
        &'a self,
        field: &str,
        key: &f64,
    ) -> Box<Iterator<Item = u64> + 'a> {
        self.get_all_indices_by_value(
            field,
            &NotNaN::new(*key).expect("NaN value is not accepted"),
        )
    }
}

pub trait RangeLookupIndexer<T> {
    fn get_index_by_range<R>(&self, field: &str, range: R) -> Option<u64>
    where
        R: RangeBounds<T>;

    fn get_all_indices_by_range<'a, R>(
        &'a self,
        field: &str,
        range: R,
    ) -> Box<Iterator<Item = u64> + 'a>
    where
        R: RangeBounds<T>;
}

macro_rules! impl_range_lookup_indexer {
    ($([$path:ident, $ty:ty])*) => {
        $(
            impl RangeLookupIndexer<$ty> for ValueIndexer {
                fn get_index_by_range<R>(&self, field: &str, range: R) -> Option<u64>
                where
                    R: RangeBounds<$ty> 
                {
                    match *self {
                        ValueIndexer::Null => None,
                        ValueIndexer::$path(ref map) => {
                            for (_, set) in map.range(range) {
                                if let Some(i) = set.get_index(0) {
                                    return Some(*i);
                                }
                            }
                            None
                        },
                        ValueIndexer::Branch(ref field_map) => field_map
                            .get(field)
                            .and_then(|entry| entry.get_index_by_range::<_>("", range)),
                        _ => panic!("Not correct type"),
                    }
                }

                fn get_all_indices_by_range<'a, R>(
                    &'a self,
                    field: &str,
                    range: R
                ) -> Box<Iterator<Item = u64> + 'a>
                where R: RangeBounds<$ty> {
                    match *self {
                        ValueIndexer::Null => Box::new(empty()),
                        ValueIndexer::$path(ref map) => Box::new(
                            map
                                .range(range)
                                .flat_map(|(_, set)| set.iter().cloned())
                        ),
                        ValueIndexer::Branch(ref field_map) => field_map
                            .get(field)
                            .map_or(
                                Box::new(empty()),
                                |entry| entry.get_all_indices_by_range::<_>("", range)
                            ),
                        _ => panic!("Not correct type"),
                    }
                }                
            }
        )*
    };
}

impl_range_lookup_indexer!{ [IntLeaf, i64] [StringLeaf, String] [FloatLeaf, NotNaN<f64>] }

impl RangeLookupIndexer<f64> for ValueIndexer {
    fn get_index_by_range<R>(&self, field: &str, range: R) -> Option<u64>
    where
        R: RangeBounds<f64>,
    {
        self.get_index_by_range(field, convert_float_range(range))
    }

    fn get_all_indices_by_range<'a, R>(
        &'a self,
        field: &str,
        range: R,
    ) -> Box<Iterator<Item = u64> + 'a>
    where
        R: RangeBounds<f64>,
    {
        self.get_all_indices_by_range(field, convert_float_range(range))
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

fn convert_float_range<R>(range: R) -> (Bound<NotNaN<f64>>, Bound<NotNaN<f64>>)
where
    R: RangeBounds<f64>,
{
    (
        convert_float_bound(range.start_bound()),
        convert_float_bound(range.end_bound()),
    )
}
