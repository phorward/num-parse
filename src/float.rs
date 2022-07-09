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

pub fn parse_float_from_iter<T: num::Float + std::str::FromStr>(chars: &mut dyn PeekableIterator<Item = char>, whitespace: bool) -> Option<T> {
    let mut has_int = false;
    let mut has_fract = false;
    let mut str = String::with_capacity(64);

    // Skip over whitespace
    if whitespace {
        while let Some(ch) = chars.peek() {
            if !ch.is_whitespace() {
                break
            }

            chars.next();
        }
    }

    // Match sign
    match chars.peek() {
        Some(ch) if *ch == '-' || *ch == '+' => {
            str.push(chars.next().unwrap());
        }
        _ => {}
    }

    // Integer part (optional)
    while let Some(ch) = chars.peek() {
        if !ch.is_numeric() {
            break
        }

        has_int = true;
        str.push(chars.next().unwrap());
    }

    // Decimal point
    match chars.peek() {
        Some(ch) if *ch == '.' => {
            str.push(chars.next().unwrap());
        }
        _ => {}
    }

    // Fractional part (optional)
    while let Some(ch) = chars.peek() {
        if !ch.is_numeric() {
            break
        }

        has_fract = true;
        str.push(chars.next().unwrap());
    }

    if !has_int && !has_fract {
        return None;
    }

    // Exponential notation
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
}

/// Parse decimal int values from a &str.
pub fn parse_float<
    T: num::Float + std::str::FromStr
>(
    s: &str,
) -> Option<T> {
    parse_float_from_iter::<T>(&mut s.chars().peekable(), true)
}

#[test]
fn test_parse_float() {
    assert_eq!(parse_float::<f64>(" -123.hello "), Some(-123f64));
}
