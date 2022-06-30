//! JavaScript-style parseInt prefix parsing of strings for Rust

use num;

/// Internal function to parse uint values from a char-iterator with a given radix.
fn parse_uint_internal<T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive>(
    chars: &mut dyn Iterator<Item = char>,
    mut radix: Option<u32>,
) -> Option<T> {
    let mut first_zero = false;
    let mut ret = T::zero();
    let mut any = false;

    while let Some(ch) = chars.next() {
        match ch {
            '0' if !first_zero => {
                first_zero = true;
            }
            'x' | 'X' if first_zero && radix.is_none() => radix = Some(16),
            dig => {
                let radix = radix.unwrap_or(10);
                match dig.to_digit(radix) {
                    Some(digit) => {
                        ret = ret.checked_mul(&T::from_u32(radix).unwrap()).unwrap();
                        ret = ret.checked_add(&T::from_u32(digit).unwrap()).unwrap();
                        any = true;
                    }
                    None => break,
                }
            }
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
    chars: &mut dyn Iterator<Item = char>,
    radix: Option<u32>,
) -> Option<T> {
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
pub fn parse_uint_from_iter<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    chars: &mut dyn Iterator<Item = char>,
) -> Option<T> {
    parse_uint_from_iter_with_radix(chars, None)
}

/// Parse int values from an iterator with a given radix.
pub fn parse_int_from_iter_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    chars: &mut dyn Iterator<Item = char>,
    radix: Option<u32>,
) -> Option<T> {
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

    if let Some(ret) = parse_uint_internal::<T>(&mut chars, radix) {
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
    chars: &mut dyn Iterator<Item = char>,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(chars, None)
}

/// Parse uint values from a &str with a given radix.
pub fn parse_uint_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive,
>(
    s: &str,
    radix: u32,
) -> Option<T> {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars(), Some(radix))
}

/// Parse decimal uint values from a &str.
pub fn parse_uint<T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive>(
    s: &str,
) -> Option<T> {
    parse_uint_from_iter_with_radix::<T>(&mut s.chars(), None)
}

/// Parse int values from a &str with a given radix.
pub fn parse_int_with_radix<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    s: &str,
    radix: u32,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(&mut s.chars(), Some(radix))
}

/// Parse decimal int values from a &str.
pub fn parse_int<
    T: num::Integer + num::CheckedAdd + num::CheckedMul + num::FromPrimitive + num::Signed,
>(
    s: &str,
) -> Option<T> {
    parse_int_from_iter_with_radix::<T>(&mut s.chars(), None)
}

#[test]
fn test_parse_uint_i64() {
    assert_eq!(parse_uint::<i64>(" 123hello "), Some(123i64));
    assert_eq!(parse_uint::<i64>(" 0xcafebabe "), Some(3405691582i64));
    assert_eq!(parse_uint::<i64>(" 0x "), None);
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
    assert_eq!(parse_int_with_radix::<i64>("  0xcafebabeyeah", 16), None);
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
    assert_eq!(parse_int_with_radix::<i64>("  -0xcafebabeyeah", 16), None);
}
