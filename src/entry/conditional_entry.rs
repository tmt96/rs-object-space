use std::iter::empty;
use std::sync::Arc;
use std::borrow::Borrow;
use std::iter::IntoIterator;
use std::collections::range::RangeArgument;

use serde_json::value::{from_value, Value};
use serde::ser::Serialize;
use serde::de::Deserialize;

use entry::helpers::{deflatten, get_all_prims_conditional};
use entry::TreeSpaceEntry;

pub trait ConditionalEntry<U> {
    fn get_conditional<R, T>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<U>;

    fn get_all_conditional<'a, R, T>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U>;

    fn remove_conditional<R, T>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<U>;

    fn remove_all_conditional<'a, R, T>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<U>;
}

impl ConditionalEntry<i64> for TreeSpaceEntry {
    fn get_conditional<R, T>(&self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<i64>,
    {
        match self.get_int_conditional_helper(field, condition) {
            Some(arc) => {
                let val: &Value = arc.borrow();
                from_value(deflatten(val.clone())).ok()
            }
            None => None,
        }
    }

    fn get_all_conditional<'a, R, T>(
        &'a self,
        field: &str,
        condition: R,
    ) -> Box<Iterator<Item = T> + 'a>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64>,
    {
        match *self {
            TreeSpaceEntry::Null => Box::new(empty()),
            TreeSpaceEntry::IntLeaf(ref int_map) => get_all_prims_conditional(int_map, condition),
            TreeSpaceEntry::Branch(ref field_map) => match field_map.get(field) {
                Some(entry) => entry.get_all_conditional("", condition),
                None => panic!("No such field found!"),
            },
            _ => panic!("Not an int type or a struct holding an int"),
        }
    }

    fn remove_conditional<R, T>(&mut self, field: &str, condition: R) -> Option<T>
    where
        for<'de> T: Serialize + Deserialize<'de>,
        R: RangeArgument<i64>,
    {
        match self.remove_int_conditional(field, condition) {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn remove_all_conditional<'a, R, T>(&'a mut self, field: &str, condition: R) -> Vec<T>
    where
        for<'de> T: Deserialize<'de> + 'static,
        R: RangeArgument<i64>,
    {
        self.remove_all_int_conditional(field, condition)
            .into_iter()
            .filter_map(|arc| match Arc::try_unwrap(arc) {
                Ok(value) => from_value(deflatten(value)).ok(),
                Err(_) => None,
            })
            .collect()
    }
}
