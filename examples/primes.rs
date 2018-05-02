extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::sync::Arc;
use std::thread;

use object_space::{ObjectSpace, RangeLookupObjectSpace, TreeObjectSpace, ValueLookupObjectSpace};

fn main() {
    let mut args = env::args();
    let upper_lim = args.nth(1)
        .and_then(|input| input.parse::<i64>().ok())
        .expect("please provide an integer input");

    let thread_count = args.next()
        .and_then(|input| input.parse::<i64>().ok())
        .unwrap_or(4);

    run(upper_lim, thread_count);
}

fn run(upper_lim: i64, thread_count: i64) {
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
        let mut current_pos = n as f64;
        let mut end = n;
        let gap = ((max - n) as f64) / (thread_count as f64);

        for _ in 0..thread_count {
            // divide work evenly between threads
            let start = end;
            current_pos = current_pos + gap;
            end = current_pos.round() as i64;

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
            clone.take_by_value::<Task>("finished", &true);
        }
        n = max;
    }

    // for i in space.read_all::<i64>() {
    //     println!("{}", i);
    // }
}

fn check_numbers(space: Arc<TreeObjectSpace>) {
    loop {
        let task = space.take_by_value::<Task>("finished", &false);
        let max = task.end;
        let min = task.start;
        let upper_limit = (max as f64).sqrt() as i64 + 1;
        let primes: Vec<i64> = space
            .read_all_by_range::<i64, _>("", ..upper_limit)
            .collect();
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
