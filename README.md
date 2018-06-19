# rs-object-space

This crate is an implementation of ObjectSpace - a natural progression on the idea of [TupleSpace](https://en.wikipedia.org/wiki/Tuple_space) proposed by Gelernter in 1985. ObjectSpace is a data structure with the capability of holding any structure (e.g: a string, an int, and a complex struct could all lives under one ObjectSpace). It also allows retrieving a struct based on the value of a field.

This crate also provides a fully thread-safe implementation of ObjectSpace, which allows simple concurrent and distributed programming.

The crate requires Rust nightly for build and usage.

## Building

You need a nightly build of [Rust](https://www.rust-lang.org/) to build the library. To install nightly Rust, first follow [these instructions](https://www.rust-lang.org/en-US/install.html) to install Rustup, then execute `rustup install nightly` via the terminal.

To build the library, run `cargo build`, or `cargo build --release` to get the fully optimized version. To run the tests of the library, run `cargo test`

To build documentation, run `cargo doc`. Documentation could be found at `target/doc/object_space/index.html`.

## Building & Running Examples

There are two examples in `examples` folder: `primes` calculate all primes up to a number, and `reminder` is a simple reminder program using ObjectSpace.

To build/run examples, do `cargo build(run) --example <example_name>`. For example: `cargo run --example reminder`

## White Paper

We provide a white paper to go along with this project. The white paper explains in further detail the inspiration and goal of this project. The content of the paper could be found at [paper/final_paper.md](paper/final_paper.md). While it is readable in its Markdown format, the paper is meant to read as a PDF file generated from Pandoc. To generate the PDF file, make sure you have Pandoc installed, `cd` to the `paper` folder and run:

```
pandoc --filter pandoc-citeproc final_paper.md -o final_paper.pdf
```

A precompiled PDF version of the paper could be found on ResearchGate with DOI 10.13140/RG.2.2.32432.05124. Notice that this version (generated June 19th 2018) could be outdated in the future.

## Issues

See the [Issues section](https://github.com/tmt96/rObjectSpace/issues) for more info on ongoing issues. Currently, most important issues are:

* [ ] Complete Agent Interface #3
* [ ] Timeout for blocking read #6
