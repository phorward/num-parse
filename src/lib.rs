//! JavaScript-style parseInt-like parsing of numbers from strings in Rust

use num;

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

/// Internal function to parse uint values from a char-iterator with a given radix.
pub fn parse_uint_internal<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    chars: &mut dyn PeekableIterator<Item = char>,
    mut radix: Option<u32>,
) -> Option<T> {
    let mut ret = T::zero();
    let mut any = false;

    if radix.is_none() {
        if let Some('0') = chars.peek() {
            chars.next();
            any = true;

            match chars.peek() {
                Some('x') | Some('X') => {
                    radix = Some(16);
                    chars.next();
                    any = false; // expecting a hex number!
                }
                _ => {}
            }
        }
    }

    let radix = radix.unwrap_or(10);

    while let Some(dig) = chars.peek() {
        match dig.to_digit(radix) {
            Some(digit) => {
                ret = ret.checked_mul(&T::from_u32(radix).unwrap()).unwrap();
                ret = ret.checked_add(&T::from_u32(digit).unwrap()).unwrap();

                chars.next();
                any = true;
            }
            None => break,
        }
    }

    if any {
        Some(ret)
    } else {
        None
    }
}

/// Parse uint values from an iterator with a given radix.
pub fn parse_uint_from_iter_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    chars: &mut dyn PeekableIterator<Item = char>,
    radix: Option<u32>,
    whitespace: bool,
) -> Option<T> {
    while let Some(ch) = chars.peek() {
        if whitespace && ch.is_whitespace() {
            chars.next();
            continue;
        }

        break;
    }

    parse_uint_internal::<T>(chars, radix)
}

/// Parse decimal uint values from an iterator.
pub fn parse_uint_from_iter<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    chars: &mut dyn PeekableIterator<Item = char>,
    whitespace: bool,
) -> Option<T> {
    parse_uint_from_iter_with_radix(chars, None, whitespace)
}

/// Parse int values from an iterator with a given radix.
pub fn parse_int_from_iter_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    chars: &mut dyn PeekableIterator<Item = char>,
    radix: Option<u32>,
    whitespace: bool,
) -> Option<T> {
    let mut neg = false;

    while let Some(ch) = chars.peek() {
        if whitespace && ch.is_whitespace() {
            chars.next();
            continue;
        }

        if *ch == '+' || *ch == '-' {
            neg = *ch == '-';
            chars.next();
        }

        break;
    }

    if let Some(ret) = parse_uint_internal::<T>(chars, radix) {
        if neg {
            Some(-ret)
        } else {
            Some(ret)
        }
    } else {
        None
    }
}

/// Parse decimal int values from an iterator.
pub fn parse_int_from_iter<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    chars: &mut dyn PeekableIterator<Item = char>,
    whitespace: bool,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(chars, None, whitespace)
}

/// Parse uint values from a &str with a given radix.
pub fn parse_uint_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    s: &str,
    radix: u32,
) -> Option<T> {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars().peekable(), Some(radix), true)
}

/// Parse decimal uint values from a &str.
pub fn parse_uint<T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive>(
    s: &str,
) -> Option<T> {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars().peekable(), None, true)
}

/// Parse int values from a &str with a given radix.
pub fn parse_int_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    s: &str,
    radix: u32,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(&mut s.chars().peekable(), Some(radix), true)
}

/// Parse decimal int values from a &str.
pub fn parse_int<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    s: &str,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(&mut s.chars().peekable(), None, true)
}

#[test]
fn test_parse_uint_i64() {
    assert_eq!(parse_uint::<i64>(" 123hello "), Some(123i64));
    assert_eq!(parse_uint::<i64>(" 0xcafebabe "), Some(3405691582i64));
    assert_eq!(parse_uint::<i64>(" 0 "), Some(0));
    assert_eq!(parse_uint::<i64>(" 0x "), None);
    assert_eq!(parse_uint::<i64>(" 0x1 "), Some(1));
    assert_eq!(parse_uint::<i64>(" 456hello "), Some(456i64));
    assert_eq!(parse_uint::<i64>(" -789hello "), None);
}

#[test]
fn test_parse_uint_base16_i64() {
    assert_eq!(
        parse_uint_with_radix::<i64>("CAFEBABE", 16),
        Some(3405691582i64)
    );
    assert_eq!(
        parse_uint_with_radix::<i64>("  cafebabeyeah", 16),
        Some(3405691582i64)
    );
    assert_eq!(parse_int_with_radix::<i64>("  0xcafebabeyeah", 16), Some(0));
}

#[test]
fn test_parse_int_i64() {
    assert_eq!(parse_int::<i64>("123hello"), Some(123i64));
    assert_eq!(parse_int::<i64>("    456hello"), Some(456i64));
    assert_eq!(parse_int::<i64>("  -789hello"), Some(-789i64));
}

#[test]
fn test_parse_int_base16_i64() {
    assert_eq!(
        parse_int_with_radix::<i64>("  -CAFEBABE", 16),
        Some(-3405691582i64)
    );
    assert_eq!(
        parse_int_with_radix::<i64>("  -cafebabeyeah", 16),
        Some(-3405691582i64)
    );
    assert_eq!(
        parse_int_with_radix::<i64>("  -0xcafebabeyeah", 16),
        Some(0)
    );
}
