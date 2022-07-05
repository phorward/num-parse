# num-parse

[![Build status](https://github.com/phorward/num-parse/actions/workflows/main.yml/badge.svg)](https://github.com/phorward/num-parse/actions/workflows/main.yml)
[![docs.rs](https://img.shields.io/docsrs/num-parse)](https://docs.rs/num-parse/latest/num_parse/)
[![crates.io](https://img.shields.io/crates/v/num-parse)](https://crates.io/crates/num-parse)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

Generic, JavaScript-like parseInt() functions for Rust.

This crate is intended to provide a fast and generic `parseInt()`-like implementation for Rust, which mostly follows the specification described in the [MDN parseInt() documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt).

Likewise in JavaScript, a `parseFloat()`-like implementation for float-types is planned as well, therefore the crate has been named `num-parse` already, althought it currently provides `parse_int()` and variative functions only.

## parse_int(), parse_uint()

`parse_int()` and `parse_uint()` are generic interfaces to parse integers from string. Whitespace in front of the parsed number is being ignored, same as anything beyond a valid number.

```rust
use num_parse::*;

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
    Some(3405691582usize)
);
```
