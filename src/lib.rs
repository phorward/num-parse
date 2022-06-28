//! JavaScript-style parseInt/parseFloat prefix parsing of strings for Rust

use num;

/// Internal function to parse uint values from a char-iterator with a given radix.
fn parse_uint_internal<T: num::PrimInt>(chars: &mut dyn Iterator<Item = char>, radix: u32) -> T {
    let mut ret = T::zero();

    while let Some(ch) = chars.next() {
        match ch.to_digit(radix) {
            Some(digit) => {
                ret = ret.checked_mul(&T::from(radix).unwrap()).unwrap();
                ret = ret.checked_add(&T::from(digit).unwrap()).unwrap();
            }
            None => break,
        }
    }

    ret
}

/// Parse uint values from an iterator with a given radix.
pub fn parse_uint_from_iter_with_radix<T: num::PrimInt>(
    chars: &mut dyn Iterator<Item = char>,
    radix: u32,
) -> T {
    let mut chars = chars.peekable();

    while let Some(ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        break;
    }

    parse_uint_internal::<T>(&mut chars, radix)
}

/// Parse decimal uint values from an iterator.
pub fn parse_uint_from_iter<T: num::PrimInt>(chars: &mut dyn Iterator<Item = char>) -> T {
    parse_uint_from_iter_with_radix(chars, 10)
}

/// Parse int values from an iterator with a given radix.
pub fn parse_int_from_iter_with_radix<T: num::PrimInt + std::ops::Neg<Output = T>>(
    chars: &mut dyn Iterator<Item = char>,
    radix: u32,
) -> T {
    let mut chars = chars.peekable();
    let mut neg = false;

    while let Some(ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        if *ch == '+' || *ch == '-' {
            neg = *ch == '-';
            chars.next();
        }

        break;
    }

    let ret = parse_uint_internal::<T>(&mut chars, radix);

    if neg {
        -ret
    } else {
        ret
    }
}

/// Parse decimal int values from an iterator.
pub fn parse_int_from_iter<T: num::PrimInt + std::ops::Neg<Output = T>>(
    chars: &mut dyn Iterator<Item = char>,
) -> T {
    parse_int_from_iter_with_radix::<T>(chars, 10)
}

/// Parse uint values from a &str with a given radix.
pub fn parse_uint_with_radix<T: num::PrimInt>(s: &str, radix: u32) -> T {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars(), radix)
}

/// Parse decimal uint values from a &str.
pub fn parse_uint<T: num::PrimInt>(s: &str) -> T {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars(), 10)
}

/// Parse int values from a &str with a given radix.
pub fn parse_int_with_radix<T: num::PrimInt + std::ops::Neg<Output = T>>(s: &str, radix: u32) -> T {
    parse_int_from_iter_with_radix::<T>(&mut s.chars(), radix)
}

/// Parse decimal int values from a &str.
pub fn parse_int<T: num::PrimInt + std::ops::Neg<Output = T>>(s: &str) -> T {
    parse_int_from_iter_with_radix::<T>(&mut s.chars(), 10)
}

#[test]
fn test() {
    assert_eq!(parse_uint::<i64>("123hello"), 123i64);
    assert_eq!(parse_uint::<i64>("    456hello"), 456i64);
    assert_eq!(parse_uint::<i64>("  -789hello"), 0i64);

    assert_eq!(parse_int::<i64>("123hello"), 123i64);
    assert_eq!(parse_int::<i64>("    456hello"), 456i64);
    assert_eq!(parse_int::<i64>("  -789hello"), -789i64);
}
