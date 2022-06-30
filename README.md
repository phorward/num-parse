# num-parse

JavaScript parseInt()-style prefix parsing of strings for Rust.

This crate is intended to provide a fast, generic, easy to use `parseInt()`-like implementation for Rust, which shall follow the specification described in the [MDN parseInt() documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt).

Likewise in JavaScript, a `parseFloat()`-like implementation for float-types is also planned, therefore the crate has been called `num-parse` already, althought it currently provides `parse_int()` and variative functions only.
