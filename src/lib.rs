// (c) 2018 Tuan Tran. Licensed under MIT License.

/*!
**NOTICE: THE LIBRARY IS STILL EXPERIMENTAL AND VERY BUGGY. API IS INCOMPLETE AND SUBJECTED TO CHANGE. PLEASE PROCEED WITH CAUTION**

This crate is an implementation of ObjectSpace - a natural progression on the idea of [TupleSpace](https://en.wikipedia.org/wiki/Tuple_space) proposed by Gelernter in 1985. ObjectSpace is a data structure with the capability of holding any structure (e.g: a string, an int, and a complex struct could all lives under one ObjectSpace). It also allows retrieving a struct based on the value of a field.

This crate also provides a fully thread-safe implementation of ObjectSpace, which allows simple concurrent and distributed programming.

# API

An ObjectSpace could perform the following tasks:
- `write` an object to the space. E.g: `space.write(test_struct)`
- `try_read` (non-blocking), `read` (blocking), and `read_all` structs of a type. E.g: `space.try_read::<TestStruct>()`
- `try_take`, `take`, and `take_all` to remove and returns struct of a type. E.g: `space.try_take::<TestStruct>()`

Notice that an ObjectSpace could hold data from any types, which means that an i64, a String, and a complex struct could all live under one space (which leads to the somewhat wordy API for retrieving items).

Additionally, by implementing `ObjectSpaceKey` and `ObjectSpaceRange`, an ObjectSpace could retrieve item based on the value of a field. Notice that the field must be a "basic" field: the type of the field must be either an int, a string, or a bool.

E.g: Given a TestStruct:

```rust
struct TestStruct {
index: i32,
property: {
    touched: bool
}
```

`space.try_take_key::<TestStruct>("property.touched", true)` will return a `TestStruct` with the value `true` for `property.touched`. `space.try_take_range::<TestStruct>("index", 2..10)` will return a `TestStruct` with the value of `index` between in the range `2..10`

For further information, please read the documentation of `ObjectSpace`, `ObjectSpaceRange`, and `ObjectSpaceKey`

# TreeObjectSpace

`TreeSpaceObject` is a referenced implementation of `ObjectSpace` trait. It is, in essence, a concurrent HashMap of `TypeId` and corresponding `Entry` for each type. Each `Entry` stores objects by serializing & flattening their structure, then put the values of basic fields in a `BTreeMap` for efficient lookup. `TreeSpaceObject` is thread-safe, which allows it to be used in concurrent and distributed settings.

# Example

Here is a program to calculate all primes up to a limit using ObjectSpace
 
```rust
extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::thread;
use std::env;
use std::sync::Arc;

use object_space::object_space::{ObjectSpace, ObjectSpaceKey, ObjectSpaceRange, TreeObjectSpace};

fn main() {
    let mut args = env::args();
    let upper_lim = args.nth(1)
        .and_then(|input| input.parse::<i64>().ok())
        .expect("please provide an integer input");

    let thread_count = args.next()
        .and_then(|input| input.parse::<i64>().ok())
        .unwrap_or(4);

    // setup. add 2 & 3 just because we can
    let mut n = 4;
    let space = Arc::new(TreeObjectSpace::new());
    space.write::<i64>(2);
    space.write::<i64>(3);

    // create 4 worker threads
    for _ in 0..thread_count {
        let space_clone = space.clone();
        thread::spawn(move || {
            check_numbers(space_clone);
        });
    }

    // continue until we hit limit
    while n < upper_lim {
        let max = if n * n < upper_lim { n * n } else { upper_lim };

        for i in 0..thread_count {
            // divide work evenly between threads
            let start =
                n + (((max - n) as f64) / (thread_count as f64) * (i as f64)).round() as i64;
            let end =
                n + (((max - n) as f64) / (thread_count as f64) * ((i + 1) as f64)).round() as i64;

            let clone = space.clone();
            clone.write(Task {
                finished: false,
                start: start,
                end: end,
            });
        }

        // "joining" threads
        for _ in 0..thread_count {
            let clone = space.clone();
            clone.take_key::<Task>("finished", &true);
        }
        n = max;
    }
}

fn check_numbers(space: Arc<TreeObjectSpace>) {
    loop {
        let task = space.take_key::<Task>("finished", &false);
        let max = task.end;
        let min = task.start;
        let primes: Vec<i64> = space.read_all::<i64>().filter(|i| i * i < max).collect();
        for i in min..max {
            if primes.iter().all(|prime| i % prime != 0) {
                space.write(i);
            }
        }
        space.write(Task {
            finished: true,
            start: min,
            end: max,
        });
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    finished: bool,
    start: i64,
    end: i64,
}
```
*/

#![feature(collections_range)]
extern crate chashmap;
extern crate num_traits;
extern crate ordered_float;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod object_space;
pub mod agent;
mod entry;
