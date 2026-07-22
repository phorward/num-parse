# num-parse

[![Build status](https://github.com/phorward/num-parse/actions/workflows/main.yml/badge.svg)](https://github.com/phorward/num-parse/actions/workflows/main.yml)
[![docs.rs](https://img.shields.io/docsrs/num-parse)](https://docs.rs/num-parse/latest/num_parse/)
[![crates.io](https://img.shields.io/crates/v/num-parse)](https://crates.io/crates/num-parse)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

Generic, JavaScript-style parseInt() and parseFloat() functions for Rust.

This crate is intended to provide a fast and generic `parseInt()`- and `parseFloat()`-like implementation for Rust, which mostly follows the specification described in the MDN documentation for [parseInt()](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt) and [parseFloat()](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseFloat).

## parse_int(), parse_uint()

`parse_int()` and `parse_uint()` are generic interfaces to parse integers from string. Whitespace in front of the parsed number is being ignored, same as anything beyond a valid number.

```rust
assert_eq!(parse_uint::<i32>("+123 as i32 "), Some(123i32));
assert_eq!(parse_int::<i32>(" -123 as i32 "), Some(-123i32));
assert_eq!(parse_uint::<i64>("+123 as i64 "), Some(123i64));
assert_eq!(parse_int::<i64>(" -123 as i64 "), Some(-123i64));

assert_eq!(parse_int::<i64>(" - 1 is invalid "), None);
assert_eq!(
    parse_uint::<u64>(" -123 as u64, parse_int() not available for this type "),
    None
);
assert_eq!(
    parse_uint::<usize>(" 0xcafebabe triggers hex-mode parsing "),
    Some(0xCAFEBABE)
);
```

## parse_float()

TODO

## PeekableIterator

This crate is required by and implemented together with the [Tokay programming language](https://tokay.dev) to parse and calculate numerical values from a `PeekableIterator`-trait, which is also defined here. A JavaScript-like numerical parsing was thought to be useful for other projects as well.
