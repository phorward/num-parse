// num-parse
// Copyright © 2023 by Jan Max Meyer, Phorward Software Technologies.
// Licensed under the MIT license. See LICENSE for more information.

/*! num-parse

    parseInt() and parseFloat() as known from JavaScript, but generic, and in Rust!
*/

mod float;
mod int;
pub use float::*;
pub use int::*;

/// Trait defining an iterator that implements a peek method on its own.
pub trait PeekableIterator: std::iter::Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}

/// Implement PeekableIterator for all Peekable<Iterator>
impl<I: std::iter::Iterator> PeekableIterator for std::iter::Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        std::iter::Peekable::peek(self)
    }
}
