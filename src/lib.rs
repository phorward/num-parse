// num-parse
// Copyright © 2022 by Jan Max Meyer, Phorward Software Technologies.
// Licensed under the MIT license. See LICENSE for more information.

/*! num-parse

    A programming language designed for ad-hoc parsing.

    Visit [https://tokay.dev](https://tokay.dev) for more information.
*/


mod parseint;
pub use parseint::*;

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
