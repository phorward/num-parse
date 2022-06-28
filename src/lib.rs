use num;

/// JavaScript parseInt-style prefix parsing

fn parse_uint_from_iter_internal<T: num::PrimInt>(chars: &mut dyn Iterator<Item = char>) -> T {
    let mut ret = T::zero();

    while let Some(ch) = chars.next() {
        match ch.to_digit(10) {
            Some(digit) => {
                ret = ret.checked_mul(&T::from(10).unwrap()).unwrap();
                ret = ret.checked_add(&T::from(digit).unwrap()).unwrap();
            }
            None => break,
        }
    }

    ret
}

pub fn parse_uint_from_iter<T: num::PrimInt>(chars: &mut dyn Iterator<Item = char>) -> T {
    let mut chars = chars.peekable();

    while let Some(ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        break
    }

    parse_uint_from_iter_internal::<T>(&mut chars)
}

pub fn parse_int_from_iter<T: num::PrimInt + std::ops::Neg<Output = T>>(
    chars: &mut dyn Iterator<Item = char>,
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

    let ret = parse_uint_from_iter_internal::<T>(&mut chars);

    if neg {
        -ret
    } else {
        ret
    }
}

pub fn parse_uint<T: num::PrimInt>(s: &str) -> T {
    parse_uint_from_iter(&mut s.chars())
}

pub fn parse_int<T: num::PrimInt + std::ops::Neg<Output = T>>(s: &str) -> T {
    parse_int_from_iter(&mut s.chars())
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
