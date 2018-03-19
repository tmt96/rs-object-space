extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::thread;
use std::env;
use std::sync::Arc;

use object_space::object_space::{ObjectSpace, ObjectSpaceKey, ObjectSpaceRange, TreeObjectSpace};

fn main() {
    let arg = env::args()
        .nth(1)
        .expect("please provide an input number")
        .parse::<i64>()
        .expect("please provide an integer input");

    // setup. add 2 & 3 just because we can
    let mut n = 4;
    let space = Arc::new(TreeObjectSpace::new());
    space.write::<i64>(2);
    space.write::<i64>(3);

    // create 4 worker threads
    for _ in 0..4 {
        let space_clone = space.clone();
        thread::spawn(move || {
            check_numbers(space_clone);
        });
    }

    // continue until we hit limit
    while n < arg {
        let max = if n * n < arg { n * n } else { arg };

        for i in 0..4 {
            // divide work evenly between 4 threads
            let start = n + (((max - n) as f64) / 4.0 * (i as f64)).round() as i64;
            let end = n + (((max - n) as f64) / 4.0 * ((i + 1) as f64)).round() as i64;

            let clone = space.clone();
            clone.write(Task {
                finished: false,
                start: start,
                end: end,
            });
        }

        // "joining" threads
        for _ in 0..4 {
            let clone = space.clone();
            clone.take_key::<Task>("finished", &true);
        }
        n = max;
    }

    // for i in space.read_all::<i64>() {
    //     println!("{}", i);
    // }
}

fn check_numbers(space: Arc<TreeObjectSpace>) {
    loop {
        let task = space.take_key::<Task>("finished", &false);
        let max = task.end;
        let min = task.start;
        let primes: Vec<i64> = space.read_all::<i64>().filter(|i| i * i < max).collect();
        for i in min..max {
            if primes.iter().all(|prime| i % prime != 0) {
                // println!("value: {}", i);
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
