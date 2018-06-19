---
title: 'ObjectSpace: An Intuitive Model for Concurrent Programming'
author: Tuan Tran
institute: Williams College
date: May 10th, 2018
geometry: "left=1.in,right=1in,top=1in,bottom=1in"
header-includes:
  - \usepackage{fullpage}
  - \usepackage{fancyhdr, graphicx, amsmath, amssymb, mathtools, enumitem, float, verbatim, url, biblatex}
  - \usepackage{setspace, tabularx}
  - \doublespacing
  - \twocolumn
numbersections: true
bibliography: reference.bib
nocite: |
  @*
csl: ieee.csl
abstract: |
    We introduce ObjectSpace, a new concurrent programming model that aims towards flexibility and simplicity. The goal of ObjectSpace is to be intuitive to programmers while having good expressiveness and scalability. It is designed to fulfill many roles in a concurrent setting, including data passing and communication between threads.

    ObjectSpace could be considered a natural evolution of Linda, introduced by Gelernter[@gele85]. It centers around a concurrent data structure that could store objects of arbitrary type. It provides atomic addition and removal of objects of arbitrary type and lets users retrieve objects based on the value of any of its field. The addition of complex data structures provides ObjectSpace with better expressiveness and type safety than Linda.

    We also provide a referential implementation of the model in Rust. The API of the model proves to be simple and intuitive, and could be generalized in many programming languages.
---
# Introduction

The rise of the age of Big Data and planetary-scale systems has put concurrent computing front and center in modern research. The most successful corporations such as Google, Microsoft, or Facebook have spent huge effort providing simple, extensible, and fault-tolerant concurrent programming models and have enjoyed considerable success [@dean08; @chan08; @isar07; @shva10]. On the consumer's side of computing, for the last ten years, chip producers have been researching on putting more processors into their systems to make up for the end of Moore's Law. Having a simple and efficient concurrent model to take advantage of this new increase in power is more crucial than ever.

However, the most common paradigm for concurrency, multi-threading, requires dealing with lock conditions, which has poor usability and scalability, and is error-prone, as each new thread presents new synchronization issues (coupled with the fact that `pthread` is not an intuitive interface). Message passing interface (MPI) requires machines to be tightly coupled in both time and space, and therefore is also difficult to use [@eugs03]. Even the low-level infrastructure provided by the Unix environment is limited in its capability. This issue leads to various studies on concurrent programming, starting from the early 1970s, including original and impactful ideas in all aspects: hardware support [@ladn80; @blel89], compiler support [@baco94], performance analysis [@cull93], and programming models.

Recently, the distributed community seems to favor ad hoc systems such as MapReduce [@dean08] or Spark [@zaha10], which focus on 'big data' processing and manipulation. While they have proven to be successful in this particular field of parallel processing, these models achieve their concurrency by limiting their API, which hampers their ability to integrate with other workflows and makes them unsuitable for a wide array of concurrent programming tasks [@rang07].

A simple and elegant approach to distributed computing is Linda which was first proposed by Gelernter[@gele85]. Although Linda has not received much attention from the community, we believe it offers a great balance between simplicity and flexibility. Linda centers around a shared memory space storing tuples from client nodes in the system. Through Linda's atomic read and write operations, agents in a concurrent system could achieve time and space decoupling. Its biggest strength though lies on its ability to allow wildcards in its operations, which enables nodes in the system to control the scope of their operations and open many possibilities for communication.

However, as our need to communicate within a concurrent setting gets more complex, Linda's design to only pass tuples between threads becomes a big limitation. Not only that tuples are too simplistic to represent structures occurred in real production, they also fail to convey the meaning of the elements in a message individually and as a whole.

ObjectSpace is an extension to Linda aiming to fix these problems. By storing complex objects instead of simple tuples, it enhances the system's power, improves its flexibility, and enforces type safety, fixing the biggest problems of Linda. We also retain Linda's unique ability to specify a value of object's field as "filter" condition, producing a powerful but intuitive framework for concurrent programming.

In the next sections of this paper, we will describe the API of an ObjectSpace and introduce a proof-of-concept implementation in Rust, a modern language for system programming gearing towards safety and concurrency. Then we will introduce an example using ObjectSpace to achieve concurrency and analyze the framework's performance. Finally, we propose a few directions which future work could focus on.

# The ObjectSpace API

## Basic Operations

The basic operations of an ObjectSpace `space` includes `write`, `read`, and `take`. Each of these operations is atomic. The `write` operation is simple: given an object `obj`, we could add it to the ObjectSpace by calling `space.write(obj)`. Notice that `obj` could be of any type: an int, a string, a boolean, a tuple, or a complex object.

`read` operation has three variations: \linebreak `space.try_read<T>()` is a non-blocking read of *one* object of type `T` from the space, `space.read<T>()` is a blocking read, and `space.read_all<T>()` is a non-blocking read of *all* elements of type `T`. Notice that since an ObjectSpace could store objects of any type, a generic type is necessary for the operation.

A `take` operation is similar to `read`, except that after an object is read, it will be destroyed from the ObjectSpace. It also has three variations as `read`.

## Conditional Operations

Beside reading objects of a type, ObjectSpace also allows filtering output objects based on the value of a field of the type. We call this *conditional operations*. There are two main types of conditional operations:

- Reading by exact value: Given a field name and a value, we return an object of which the specified field has the given value. For example: `space.read_by_value<T>("age", 8)` reads an object of type `T` whose field `'age'` has value 9.
- Reading by range: Given a field name and a range of values, we return an object of which the value of the specified field is in the specified range. For example: `space.read_by_range<T>("age", range(6, 9))` reads an object of type `T` whose filed `'age'` has value between 6 and 9.

Each of these types of conditional operations has all of the variations found in normal `read` and `take` operations.

In general, the surface API of ObjectSpace is small and very easy to understand. However, it provides a robust base for multiple data passing and communication jobs for concurrent programming.

# Reference Implementation

We provide a reference implementation of the ObjectSpace model. The implementation is MIT and Apache dual-licensed (as is the standard in the Rust community) and could be found on GitHub [@tmt18]. This implementation is written in Rust, a language for system programming focusing on safety and concurrency. Since Rust forbids a lot of possible concurrent error through its concepts of lifetime and ownership and a very thorough compiler, limiting how variables could be created and passed between functions, it has a high initial learning curves. However, the benefit coming from native performance with high-level features, and straightforward concurrent programming proves worth these limitation.

The main data structure of our implementation is a `HashMap` between the type's ID and an `Entry` storing all objects of the corresponding type. Before being written into the ObjectSpace, objects are serialized into a flatten JSON-like structure and assigned a unique ID. The `Entry` for each type consists of two data structures: a `HashMap` between an object's ID and itself; and a reversed indexer which maps a possible value of a field with an ID list of objects whose mentioned field has the corresponding value.

This structure enables straightforward implementation of all of ObjectSpace's features, especially conditional operations. A downside of this implementation though is that it can only store objects that are JSON-serializable. Moreover, conditional operations could only operate on "leaf fields" of an object: fields whose values are numbers, booleans, or strings; and conditional operations on more than one field at a time are complicated. However, we find that despite these operations, our implementation is still very robust and flexible, and proves to be suitable for a wide array of concurrent programming jobs.

# Example: Calculating Primes

An example of ObjectSpace usage could be found in \ref{code-appendix}. This example calculates and prints out all prime numbers smaller than ten million. The code requires a few changes compared to a program written for single-threaded setting, but in general still clear and simple to follow.

This example introduces two common usages of ObjectSpace. The more obvious usage is for data passing: after a worker thread finds a prime number, it writes the number into the ObjectSpace. Then at the end of the program, the master thread reads in all calculated prime numbers from the ObjectSpace and prints them out.

The second usage of ObjectSpace is communication, achieved through the `Task` object, which represents a range of number. In each round of iteration, the master thread writes new `Tasks` to the ObjectSpace. Each worker thread will take one `Task` and calculate the prime numbers within the range of the `Task`, before writing them to the ObjectSpace and using the `Task` to communicate back to the master that a job has been finished. Notice that the master thread does not need to know which task is taken by which thread, it just needs to know that such task has been completed (this information, however, could be easily added to a `Task` by the programmer if necessary). This helps us achieve space decoupling between threads and reduce the complexity of the program.

We also provide a few other examples of ObjectSpace, including a reminder program in the same vein as one introduced in Gelernter's original paper[@gele85], and a program for drawing Mandelbrot fractal. All of them could be found in our GitHub repository[@tmt18].

## Performance

Here we measure the performance of the aforementioned prime numbers example to test the scalability of the framework. All experiments are done on a MacBook Pro 15 inch late 2016 running macOS 10.13, with Intel i7 6820HQ, 16GB of RAM and 512 GB of SSD.

\begin{tabularx}{\columnwidth}{|X||c|}
\hline
Setup & Time(s)\\
\hline
Normal single-threaded & 24.193\\
ObjectSpace 1 thread & 32.248\\
ObjectSpace 2 threads & 19.507\\
ObjectSpace 4 threads & 13.931\\
ObjectSpace 8 threads & 14.741\\
ObjectSpace 16 threads & 16.084\\
ObjectSpace 32 threads & 16.731\\
\hline
\end{tabularx}

Notice that since this machine has only 4 cores, 8 threads (and several already used to run the OS), we expect slowdown as the number of threads exceeds four.

The experiment shows that our implementation of ObjectSpace introduces a non-trivial overhead to the program, most likely due to serialization and synchronization mechanism using the `Task` structure, which is not necessary for this case. However, it proves to scale well up to the number of available threads in the system. We believe that given more optimization in the example program, we could achieve even better scalability.

# Insights and Future Research

The design goal of ObjectSpace API is based on three principles: simplicity, flexibility, and good language integration. The API surface of ObjectSpace is small and consistent, lowering the learning curve while allowing a high degree of flexibility for the programmers to express their intention through conditional operations. In practice, we find that ObjectSpace is suitable for a wide array of concurrent work. The integration of the reference implementation with the Rust language makes it very natural to learn and use. We expect any future implementation of the paradigm to integrate with their respective language similarly closely.

Compared to MapReduce, another popular concurrent programming framework[@dean08], ObjectSpace does not force the programmers into any particular paradigm, instead merely serving as a facilitator for parallel computing. Its flexible nature means that it could serve as either a data storage, data passer, or communication intermediary. As a result, programmers have more freedom in choosing the best design for their program; yet it also requires more thought and effort on the part of the programmers to get their model correctly. However, a MapReduce-like paradigm could be achieved quite easily through ObjectSpace using an interface similar to a `Task` in our example.

When using ObjectSpace, programmers need to carefully think about the flow of objects passing into the structure to maximize performance. We, however, do not think this is a fault of the paradigm, since parallel programming is too complex to hide perfectly, and thus it is best to require programmers to consider it explicitly[@isar07].

Due to the experimental nature of the framework, there are still a lot of room for improvement. Most obvious of all is performance enhancement: the proof-of-concept implementation still heavily relies on locks for the sake of ease of implementation, which brings a lot of overhead to the program. Serialization of objects, which in some cases is unnecessary, also contributes to the overall overhead. We hope to improve in future iterations.

We would also like to investigate additional APIs that could benefit the framework. An example of such an API is the ability to declare conditional operations on multiple fields at once, mirroring such ability in Linda. Extra work still needs to be done to figure the easiest way to implement such an API.

A big goal of ObjectSpace is to generalize to the distributed setting, similar to Linda. Since in our implementation, objects are already serialized before added to the ObjectSpace, adding distributed capability should be possible. The distributed setting will bring new unique challenges to ObjectSpace, for example: whether objects in he framework should be stored distributively or centrally.

# Acknowledgement

This project could not have been completed without help and support from Professor Duane Bailey and Professor Jeannie Albretch, who have directly supervised this project.

Special thanks to my friend Daishiro Nishida, who has worked with me on the first prototype implemented in C#, with could be found at https://github.com/tmt96/dotSpace-objectSpace.

# References

<div id="refs"></div>

\appendix
\onecolumn

# Calculating Prime Numbers with ObjectSpace \label{code-appendix}

```rust
extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::thread;
use std::env;
use std::sync::Arc;

use object_space::{ObjectSpace, ValueLookupObjectSpace, RangeLookupObjectSpace, TreeObjectSpace};

fn main() {
    let mut args = env::args();
    let upper_lim = 1000000;
    let thread_count = 4;

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
            clone.take_by_value::<Task>("finished", &true);
        }
        n = max;
    }
}

fn check_numbers(space: Arc<TreeObjectSpace>) {
    loop {
        let task = space.take_by_value::<Task>("finished", &false);
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
