//! JavaScript-style parseFloat-like parsing of numbers from strings in Rust.

/*
This is a naive approach for a parse_float() that was implemented in Tokay already.

It just checks for a valid character order, collects each character and afterwards uses String::parse<T>() to
parse the specific string. This isn't nice. A better approach worth for num-parse would be to parse the characters
collected directly into a float.

A solution for this can already be found in https://doc.rust-lang.org/src/core/num/dec2flt/parse.rs.html.
Maybe we can borrow this and use it together with a PeekableIterator. It should not be as performant like with
std::num, but use a similar algorithm.
*/

use super::*;
use num;

pub fn parse_float_from_iter<T: num::Float + num::FromPrimitive>(
    chars: &mut dyn PeekableIterator<Item = char>,
    whitespace: bool,
) -> Option<T> {
    let mut neg = false;
    let mut int: Option<u32> = None;
    let mut dec: Option<u32> = None;

    // Skip over whitespace
    if whitespace {
        while let Some(ch) = chars.peek() {
            if !ch.is_whitespace() {
                break;
            }

            chars.next();
        }
    }

    // Match sign
    match chars.peek() {
        Some(ch) if *ch == '-' || *ch == '+' => {
            neg = chars.next().unwrap() == '-';
        }
        _ => {}
    }

    // Integer part (optional)
    while let Some(dig) = chars.peek() {
        int = match dig.to_digit(10) {
            Some(digit) => {
                let mut int = int.unwrap_or(0);

                int = int.checked_mul(10).unwrap();
                int = int.checked_add(digit).unwrap();

                chars.next();
                Some(int)
            }
            None => break,
        }
    }

    // Decimal point (mandatory)
    match chars.peek() {
        Some(ch) if *ch == '.' => {
            chars.next();
        }
        _ => return None,
    }

    // Decimal part (optional)
    while let Some(dig) = chars.peek() {
        dec = match dig.to_digit(10) {
            Some(digit) => {
                let mut dec = dec.unwrap_or(0);

                dec = dec.checked_mul(10).unwrap();
                dec = dec.checked_add(digit).unwrap();

                chars.next();
                Some(dec)
            }
            None => break,
        }
    }

    if int.is_some() || dec.is_some() {
        let int = T::from_u32(int.unwrap_or(0)).unwrap();
        let dec = T::from_u32(dec.unwrap_or(0)).unwrap();
        let ten = T::from_u32(10).unwrap();

        let dec = dec
            / num::pow::pow(
                ten,
                std::iter::successors(Some(dec), |&n| (n >= ten).then(|| n / ten)).count(),
            );

        let ret = int + dec;

        return if neg { Some(-ret) } else { Some(ret) };
    }

    None

    // Exponential notation
    /*
    match chars.peek() {
        Some('e') | Some('E') => {
            let mut exp = String::with_capacity(10);
            exp.push(chars.next().unwrap());

            let mut have_sign = false;
            let mut have_digs = false;
            while let Some(ch) = chars.peek() {
                match ch {
                    '+' | '-' if !have_sign => {
                        have_sign = true;
                    }
                    ch if ch.is_numeric() => {
                        have_digs = true;
                    }
                    _ => break
                }

                exp.push(chars.next().unwrap());
            }

            if have_digs {
                str.push_str(&exp);
            }
        }
        _ => {}
    }

    str.parse::<T>().ok()
    */
}

/// Parse decimal int values from a &str.
pub fn parse_float<T: num::Float + num::FromPrimitive>(s: &str) -> Option<T> {
    parse_float_from_iter::<T>(&mut s.chars().peekable(), true)
}

#[test]
fn test_parse_float() {
    assert_eq!(parse_float::<f64>(" -123.hello "), Some(-123f64));
    assert_eq!(parse_float::<f64>(" -13.37.hello "), Some(-13.37f64));
}
