# darc - Dynamically-atomic reference-counting pointers

[![Crate](https://img.shields.io/crates/v/darc.svg)](https://crates.io/crates/darc)
[![Documentation](https://docs.rs/darc/badge.svg)](https://docs.rs/darc)
![minimum rustc 1.31](https://img.shields.io/badge/rustc-1.31+-red.svg)

This is a proof of concept of a Rust `Rc<T>` type that can *dynamically* choose
whether to use atomic access to update its reference count. A related `Arc<T>`
can be created which offers thread-safe (`Send + Sync`) access to the same
data. If there's never an `Arc`, the `Rc` never pays the price for atomics.

`darc` currently requires `rustc 1.31.0` or greater.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
