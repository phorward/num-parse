/*! Generic, JavaScript-like parseFloat() function for parsing floating point numbers
from any character-emitting resource. */

use super::*;
use num;

/** Parse float values from a PeekableIterator.

Trailing `whitespace` is accepted, when set to `true`.
*/
pub fn parse_float_from_iter<T: num::Float + num::FromPrimitive + std::fmt::Display>(
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

                int = int * 10 + digit;

                chars.next();
                Some(int)
            }
            None => break,
        }
    }

    // Decimal point (this *is* mandatory!)
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

                dec = dec * 10 + digit;

                chars.next();
                Some(dec)
            }
            None => break,
        }
    }

    // Either integer or decimal part must be given, otherwise reject
    if int.is_none() && dec.is_none() {
        return None;
    }

    // Turn integer and decimal part into floating point number
    let int = T::from_u32(int.unwrap_or(0)).unwrap();
    let dec = T::from_u32(dec.unwrap_or(0)).unwrap();
    let ten = T::from_u32(10).unwrap();

    let mut precision =
        std::iter::successors(Some(dec), |&n| (n >= ten).then(|| n / ten)).count() as u32;
    let mut ret = int + dec / ten.powi(precision as i32);

    // Parse optionally provided exponential notation
    match chars.peek() {
        Some('e') | Some('E') => {
            chars.next();

            let mut neg = false;

            match chars.peek() {
                Some(ch) if *ch == '-' || *ch == '+' => {
                    neg = chars.next().unwrap() == '-';
                }
                _ => {}
            }

            let mut exp: u32 = 0;

            while let Some(dig) = chars.peek() {
                match dig.to_digit(10) {
                    Some(digit) => {
                        exp = exp * 10;
                        exp = exp + digit;

                        chars.next();
                    }
                    None => break,
                }
            }

            if neg {
                precision += exp;
            } else if precision < exp {
                precision = 0;
            }

            // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db0408483b26c89505a8ec2be4f57f42
            for _ in 0..exp {
                if neg {
                    ret = ret / ten;
                }
                else {
                    ret = ret * ten;
                }
            }
        }
        _ => {}
    }

    let factor = ten.powf(T::from_u32(precision).unwrap());

    //println!("before ret = {}, precision = {} factor = {}", ret, precision, factor);
    ret = (ret * factor).round() / factor;

    // Negate when necessary
    if neg {
        Some(-ret)
    } else {
        Some(ret)
    }

    // 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001337
    // 0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000013369999999999999
}

/// Parse float values from a &str, ignoring trailing whitespace.
pub fn parse_float<T: num::Float + num::FromPrimitive + std::fmt::Display>(s: &str) -> Option<T> {
    parse_float_from_iter::<T>(&mut s.chars().peekable(), true)
}

#[test]
fn test_parse_float_f32() {
    assert_eq!(parse_float::<f32>(" -123.hello "), Some(-123f32));
    assert_eq!(parse_float::<f32>(" -13.37.hello "), Some(-13.37f32));
    assert_eq!(parse_float::<f32>(" -13.37e2.hello "), Some(-1337f32));
    assert_eq!(parse_float::<f32>(" -13.37e-2.hello "), Some(-0.1337f32));
    assert_eq!(
        parse_float::<f32>(" -13.37e-16 "),
        Some(-0.000000000000001337f32)
    );
    assert_eq!(parse_float::<f32>(" -1337.0e-30f32 "), Some(-1337.0e-30f32));
    /*
    assert_eq!(parse_float::<f32>(" -1337.0e-300f32 "), Some(-1337.0e-300f32));
    */
}

#[test]
fn test_parse_float_f64() {
    assert_eq!(parse_float::<f64>(" -123.hello "), Some(-123f64));
    assert_eq!(parse_float::<f64>(" -13.37.hello "), Some(-13.37f64));
    assert_eq!(parse_float::<f64>(" -13.37e2.hello "), Some(-1337f64));
    assert_eq!(parse_float::<f64>(" -13.37e-2.hello "), Some(-0.1337f64));
    assert_eq!(
        parse_float::<f64>(" -13.37e-16 "),
        Some(-0.000000000000001337f64)
    );
    assert_eq!(parse_float::<f64>(" -1337.0e-30f64 "), Some(-1337.0e-30f64));
    assert_eq!(parse_float::<f64>(" -1337.0e-300f64 "), Some(-1337.0e-300f64));
    /*
    assert_eq!(
        parse_float::<f32>(" -1337.0e-326f32 "),
        Some(-1337.0e-326f32)
    );
    */
}
