use std::borrow::Borrow;
use std::ops::RangeBounds;
use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::value::Value;

pub mod indexer;

use entry::indexer::{RangeLookupIndexer, ValueIndexer, ValueLookupIndexer};

pub struct Entry {
    counter: u64,
    value_map: IndexMap<u64, Arc<Value>>,
    indexer: ValueIndexer,
}

impl Entry {
    pub fn new() -> Self {
        Entry {
            counter: 0,
            value_map: IndexMap::new(),
            indexer: ValueIndexer::new(),
        }
    }

    pub fn add(&mut self, obj: Value) {
        self.add_value_to_list(Arc::new(obj.clone()));
        self.indexer.add(obj, self.counter)
    }

    pub fn get(&self) -> Option<Value> {
        self.value_map.values().next().map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }

    pub fn get_all<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
        Box::new(self.value_map.values().map(|item| {
            let val: &Value = item.borrow();
            val.clone()
        }))
    }

    pub fn remove(&mut self) -> Option<Value> {
        self.value_map.pop().map(|(key, value)| {
            let val: &Value = value.borrow();
            self.indexer.remove(key, val);
            val.clone()
        })
    }

    pub fn remove_all(&mut self) -> Vec<Value> {
        let result = self.get_all().collect();
        *self = Entry::new();
        result
    }

    fn add_value_to_list(&mut self, arc: Arc<Value>) {
        self.counter += 1;
        self.value_map.entry(self.counter).or_insert(arc);
    }

    fn get_value_from_index(&self, index: &u64) -> Option<Value> {
        self.value_map.get(index).map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }

    fn remove_value_from_index(&mut self, index: &u64) -> Option<Value> {
        self.value_map.remove(index).map(|arc| {
            let val: &Value = arc.borrow();
            val.clone()
        })
    }
}

pub trait ValueLookupEntry<U> {
    fn get_by_value(&self, field: &str, key: &U) -> Option<Value>;

    fn get_all_by_value<'a>(&'a self, field: &str, key: &U) -> Box<Iterator<Item = Value> + 'a>;

    fn remove_by_value(&mut self, field: &str, key: &U) -> Option<Value>;

    fn remove_all_by_value(&mut self, field: &str, key: &U) -> Vec<Value>;
}

macro_rules! impl_value_lookup_entry {
    ($($ty:ty)*) => {
        $(            
            impl ValueLookupEntry<$ty> for Entry {
                fn get_by_value(&self, field: &str, key: &$ty) -> Option<Value> {
                    let index = self.indexer.get_index_by_value(field, key);
                    index.and_then(|i| self.get_value_from_index(&i))
                }

                fn get_all_by_value<'a>(&'a self, field: &str, key: &$ty) -> Box<Iterator<Item = Value> + 'a> {
                    let indices = self.indexer.get_all_indices_by_value(field, key);
                    Box::new(
                        indices.filter_map(move |i| self.get_value_from_index(&i))
                    )
                }

                fn remove_by_value(&mut self, field: &str, key: &$ty) -> Option<Value> {
                    let index = self.indexer.get_index_by_value(field, key);
                    index.and_then(|i| {
                        let val = self.remove_value_from_index(&i);
                        val.clone().map(|val| self.indexer.remove(i, &val));
                        val
                    })
                }

                fn remove_all_by_value(&mut self, field: &str, key: &$ty) -> Vec<Value> {
                    let indices: Vec<u64> = self.indexer.get_all_indices_by_value(field, key).collect();
                    let mut result = Vec::new();
                    for i in indices {
                        if let Some(val) = self.remove_value_from_index(&i) {
                            self.indexer.remove(i, &val.clone());
                            result.push(val);
                        }
                    }
                    result
                }
            }
        )*
    };
}

impl_value_lookup_entry!{i64 String bool f64}

pub trait RangeLookupEntry<U> {
    fn get_by_range(&self, field: &str, range: impl RangeBounds<U>) -> Option<Value>;

    fn get_all_by_range<'a>(&'a self, field: &str, range: impl RangeBounds<U>) -> Box<Iterator<Item = Value> + 'a>;

    fn remove_by_range(&mut self, field: &str, range: impl RangeBounds<U>) -> Option<Value>;

    fn remove_all_by_range<'a>(&'a mut self, field: &str, range: impl RangeBounds<U>) -> Vec<Value>;
}

macro_rules! impl_range_lookup_entry {
    ($($ty:ty)*) => {
        $(            
            impl RangeLookupEntry<$ty> for Entry {
                fn get_by_range(&self, field: &str, range: impl RangeBounds<$ty>) -> Option<Value> 
                {
                    let index = self.indexer.get_index_by_range(field, range);
                    index.and_then(|i| self.get_value_from_index(&i))
                }

                fn get_all_by_range<'a>(&'a self, field: &str, range: impl RangeBounds<$ty>) -> Box<Iterator<Item = Value> + 'a> 
                
                {
                    let indices = self.indexer.get_all_indices_by_range(field, range);
                    Box::new(
                        indices.filter_map(move |i| self.get_value_from_index(&i))
                    )
                }

                fn remove_by_range(&mut self, field: &str, range: impl RangeBounds<$ty>) -> Option<Value> 
                
                {
                    let index = self.indexer.get_index_by_range(field, range);
                    index.and_then(|i| {
                        let val = self.remove_value_from_index(&i);
                        val.clone().map(|val| self.indexer.remove(i, &val));
                        val
                    })
                }

                fn remove_all_by_range(&mut self, field: &str, range: impl RangeBounds<$ty>) -> Vec<Value> 
                
                {
                    let indices: Vec<u64> = self.indexer.get_all_indices_by_range(field, range).collect();
                    let mut result = Vec::new();
                    for i in indices {
                        if let Some(val) = self.remove_value_from_index(&i) {
                            self.indexer.remove(i, &val.clone());
                            result.push(val);
                        }
                    }
                    result
                }
            }
        )*
    };
}

impl_range_lookup_entry!{i64 String f64}
