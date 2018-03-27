# rObjectSpace

An object store library for highly concurrent program written in Rust.

To get documentation, run `cargo doc --open --no-deps`.

## Building

You need a nightly build of [Rust](https://www.rust-lang.org/) to build the library. To install nightly Rust, first follow [these instructions](https://www.rust-lang.org/en-US/install.html) to install Rustup, then execute `rustup install nightly` via the terminal.

To build the library, run `cargo build`, or `cargo build --release` to get the fully optimized version. To run the tests of the library, run `cargo test`

To build documentation, run `cargo doc`. Documentation could be found at `target/doc/object_space/index.html`.

## Building & Running Examples

There are two examples in `examples` folder: `primes` calculate all primes up to a number, and `reminder` is a simple reminder program using ObjectSpace.

To build/run examples, do `cargo build(run) --example <example_name>`. For example: `cargo run --example reminder`

## Issues

See the [Issues section](https://github.com/tmt96/rObjectSpace/issues) for more info on ongoing issues. Currently, most important issues are:

* [ ] Range & Value filtering for float fields #1
* [ ] Enum Support #2
* [ ] Complete Agent Interface #3
