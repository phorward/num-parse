/*! Generic, JavaScript-parseFloat()-like function for parsing floating point numbers
from any character-emitting resource. */

use super::*;
use num;
use num::bigint::BigUint;
use num::{ToPrimitive, Zero};

/** Number of bits (including the implicit leading bit) that `T` can hold in its
    mantissa, i.e. the largest `p` for which every integer up to `2^p` is exactly
    representable in `T`. Determined at runtime so this works for any `num::Float`
    implementation, not just `f32`/`f64`.
*/
fn mantissa_bits<T: num::Float + num::FromPrimitive>() -> u32 {
    let mut n: u32 = 1;

    while n < u64::BITS {
        let v = T::from_u64(1u64 << n).unwrap();

        if v + T::one() == v {
            return n;
        }

        n += 1;
    }

    n
}

/// Compute `10^exp` as an exact, arbitrary-precision integer.
fn pow10(exp: u64) -> BigUint {
    num::pow(BigUint::from(10u32), exp as usize)
}

/** The smallest (most negative) binary exponent of a normal (non-subnormal)
    value of `T`, i.e. `E` such that `T::min_positive_value() == 2^E`.
    Determined at runtime, like `mantissa_bits`, so this works generically.
*/
fn min_exponent<T: num::Float + num::FromPrimitive>() -> i64 {
    let min_normal = T::min_positive_value();
    let two = T::from_u32(2).unwrap();
    let mut v = T::one();
    let mut n: i64 = 0;

    while v > min_normal {
        v = v / two;
        n += 1;
    }

    -n
}

/** Divide `numerator / denominator` (both exact, non-negative integers) and
    return `(quotient, remainder, shift)` so that `quotient` has exactly
    `target_bits` bits, where `shift` records how far `numerator` (if
    positive) or `denominator` (if negative) was shifted to get there.

    `target_bits` must be positive; the bit-length estimate used to pick the
    initial shift can be off by one, so this retries (still exact, no
    precision lost) until the quotient's bit length matches exactly.
*/
fn divide_to_bits(
    numerator: &BigUint,
    denominator: &BigUint,
    target_bits: i64,
) -> (BigUint, BigUint, i64) {
    let mut shift = target_bits - (numerator.bits() as i64 - denominator.bits() as i64);

    loop {
        let n = if shift >= 0 {
            numerator << (shift as u64)
        } else {
            numerator.clone()
        };
        let d = if shift < 0 {
            denominator << ((-shift) as u64)
        } else {
            denominator.clone()
        };

        let q = &n / &d;
        let qbits = q.bits() as i64;

        if qbits == target_bits {
            let r = &n - &q * &d;
            return (q, r, shift);
        }

        // This is exact integer arithmetic, so redoing it costs nothing in
        // precision, only a cheap extra division.
        shift += target_bits - qbits;
    }
}

/** Compute `2^n` in `T`, for any (possibly negative) `n`, without ever
    forming an intermediate value outside `T`'s representable range when the
    final result would in fact be representable.

    A plain `powi` for negative exponents is typically implemented as
    `1 / base.powi(-n)` at runtime, which first computes `2^-n` -- and that
    intermediate can overflow to infinity even when `2^n` itself is a
    perfectly representable (if tiny/subnormal) value, collapsing the final
    result to zero. Halving the exponent recursively keeps every intermediate
    magnitude proportionally small, sidestepping that.
*/
fn pow2<T: num::Float + num::FromPrimitive>(n: i64) -> T {
    match n {
        0 => T::one(),
        1 => T::from_u32(2).unwrap(),
        -1 => T::one() / T::from_u32(2).unwrap(),
        n => {
            let half = n / 2;
            pow2::<T>(half) * pow2::<T>(n - half)
        }
    }
}

/** Convert a decimal number, given as an exact integer mantissa and a base-10
    exponent (`mantissa * 10^exponent`), into the nearest representable `T`.
*/
fn decimal_to_float<T: num::Float + num::FromPrimitive>(mantissa: BigUint, exponent: i64) -> T {
    if mantissa.is_zero() {
        return T::zero();
    }

    let precision = mantissa_bits::<T>() as i64;

    // Represent the exact value as a ratio of non-negative integers.
    let (base_numerator, base_denominator) = if exponent >= 0 {
        (&mantissa * pow10(exponent as u64), BigUint::from(1u32))
    } else {
        (mantissa, pow10((-exponent) as u64))
    };

    // Smallest binary exponent of a subnormal value: below this, `T` can no
    // longer hold `precision` significant bits -- precision degrades
    // gradually down to zero, exactly as IEEE 754 gradual underflow defines.
    let subnormal_floor = min_exponent::<T>() - precision + 1;

    // First pass: assume we're in the normal range and extract `precision`
    // significant bits plus one round bit.
    let (quotient, remainder, shift) =
        divide_to_bits(&base_numerator, &base_denominator, precision + 1);
    let top_exponent = (1 - shift) + precision - 1;

    let (quotient, remainder, shift) = if top_exponent < subnormal_floor - 1 {
        // Magnitude is at or below half the smallest subnormal: rounds to zero.
        (BigUint::from(0u32), BigUint::from(1u32), 0)
    } else if top_exponent < min_exponent::<T>() {
        // Subnormal range: fewer than `precision` bits are actually
        // representable here. Redo the exact division targeting exactly as
        // many bits as remain down to `subnormal_floor`, so the final
        // power-of-two scale factor stays within `T`'s representable range
        // instead of vanishing to zero on its own before being combined with
        // the mantissa.
        let bits = top_exponent - subnormal_floor + 2;
        divide_to_bits(&base_numerator, &base_denominator, bits)
    } else {
        (quotient, remainder, shift)
    };

    // `quotient` holds the candidate mantissa plus one round bit at the
    // bottom; `remainder` (relative to the divisor used for `quotient`)
    // tells us whether anything non-zero was truncated below that.
    let one = BigUint::from(1u32);
    let round_bit = &quotient & &one;
    let sticky = !remainder.is_zero();

    let mut mantissa_bits_val = &quotient >> 1u32;
    let mut bin_exp = 1 - shift;

    let round_up = if round_bit.is_zero() {
        false
    } else if sticky {
        true
    } else {
        // Exactly halfway between two representable values: round to even.
        &mantissa_bits_val & &one == one
    };

    if round_up {
        mantissa_bits_val += &one;

        // Rounding up may have carried into an extra bit; renormalize.
        if mantissa_bits_val.bits() as i64 > precision {
            mantissa_bits_val = &one << (precision as u64 - 1);
            bin_exp += 1;
        }
    }

    T::from_u64(mantissa_bits_val.to_u64().unwrap()).unwrap() * pow2::<T>(bin_exp)
}

/** Internal digit & exponent parser shared by `parse_ufloat_from_iter` and
    `parse_float_from_iter`.
*/
fn parse_ufloat_internal<T: num::Float + num::FromPrimitive>(
    chars: &mut dyn PeekableIterator<Item = char>,
) -> Option<T> {
    let mut any_digit = false;

    // Integer and decimal digits are accumulated into a single, exact,
    // arbitrary-precision mantissa (no digit is ever lost to overflow).
    let mut mantissa = BigUint::from(0u32);
    let mut frac_digits: i64 = 0;

    // Integer part (optional)
    while let Some(dig) = chars.peek() {
        match dig.to_digit(10) {
            Some(digit) => {
                mantissa = mantissa * 10u32 + digit;
                any_digit = true;
                chars.next();
            }
            None => break,
        }
    }

    // Decimal point (optional)
    if let Some('.') = chars.peek() {
        chars.next();
    }

    // Decimal part (optional)
    while let Some(dig) = chars.peek() {
        match dig.to_digit(10) {
            Some(digit) => {
                mantissa = mantissa * 10u32 + digit;
                frac_digits += 1;

                any_digit = true;
                chars.next();
            }
            None => break,
        }
    }

    // Either integer or decimal part must be given, otherwise reject
    if !any_digit {
        return None;
    }

    let mut exponent: i64 = -frac_digits;

    // Parse optionally provided exponential notation
    match chars.peek() {
        Some('e') | Some('E') => {
            chars.next();

            let mut exp_neg = false;

            match chars.peek() {
                Some(ch) if *ch == '-' || *ch == '+' => {
                    exp_neg = chars.next().unwrap() == '-';
                }
                _ => {}
            }

            let mut exp: i64 = 0;

            while let Some(dig) = chars.peek() {
                match dig.to_digit(10) {
                    Some(digit) => {
                        exp = exp * 10 + digit as i64;
                        chars.next();
                    }
                    None => break,
                }
            }

            exponent += if exp_neg { -exp } else { exp };
        }
        _ => {}
    }

    Some(decimal_to_float::<T>(mantissa, exponent))
}

/** Parse unsigned (non-negative) float values from a PeekableIterator.

Preceding `whitespace` is accepted, when set to `true`. A leading `+` is
accepted, but a `-` sign is rejected -- just like `parse_uint()` behaves
compared to `parse_int()`.
*/
pub fn parse_ufloat_from_iter<T: num::Float + num::FromPrimitive + std::fmt::Display>(
    chars: &mut dyn PeekableIterator<Item = char>,
    whitespace: bool,
) -> Option<T> {
    while let Some(ch) = chars.peek() {
        if whitespace && ch.is_whitespace() {
            chars.next();
            continue;
        } else if *ch == '+' {
            chars.next();
        }

        break;
    }

    parse_ufloat_internal::<T>(chars)
}

/** Parse float values from a PeekableIterator.

Preceding `whitespace` is accepted, when set to `true`.
*/
pub fn parse_float_from_iter<T: num::Float + num::FromPrimitive + std::fmt::Display>(
    chars: &mut dyn PeekableIterator<Item = char>,
    whitespace: bool,
) -> Option<T> {
    let mut neg = false;

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

    let ret = parse_ufloat_internal::<T>(chars)?;

    // Negate when necessary
    if neg { Some(-ret) } else { Some(ret) }
}

/// Parse unsigned (non-negative) float values from a &str, ignoring trailing whitespace.
pub fn parse_ufloat<T: num::Float + num::FromPrimitive + std::fmt::Display>(s: &str) -> Option<T> {
    parse_ufloat_from_iter::<T>(&mut s.chars().peekable(), true)
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

    // The decimal point is optional
    assert_eq!(parse_float::<f32>(" 123 "), Some(123f32));
    assert_eq!(parse_float::<f32>(" -123 "), Some(-123f32));
    assert_eq!(parse_float::<f32>(" 123e2 "), Some(12300f32));
    assert_eq!(parse_float::<f32>(" 0 "), Some(0f32));
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
    assert_eq!(
        parse_float::<f64>(" -1337.0e-296f64 "),
        Some(-1337.0e-296f64)
    ); // OK
    assert_eq!(
        parse_float::<f64>(" -1337.0e-297f64 "),
        Some(-1337.0e-297f64)
    );
    assert_eq!(
        parse_float::<f64>(" -1337.0e-298f64 "),
        Some(-1337.0e-298f64)
    );
    assert_eq!(
        parse_float::<f64>(" -1337.0e-299f64 "),
        Some(-1337.0e-299f64)
    );
    assert_eq!(
        parse_float::<f64>(" -1337.0e-300f64 "),
        Some(-1337.0e-300f64)
    );
    assert_eq!(
        parse_float::<f64>(" -1337.0e-301f64 "),
        Some(-1337.0e-301f64)
    );

    // The decimal point is optional
    assert_eq!(parse_float::<f64>(" 123 "), Some(123f64));
    assert_eq!(parse_float::<f64>(" -123 "), Some(-123f64));
    assert_eq!(parse_float::<f64>(" 123e10 "), Some(123e10f64));
    assert_eq!(parse_float::<f64>(" 0 "), Some(0f64));
}

#[test]
fn test_parse_ufloat_f32() {
    assert_eq!(parse_ufloat::<f32>(" 123.hello "), Some(123f32));
    assert_eq!(parse_ufloat::<f32>(" +123.hello "), Some(123f32));
    assert_eq!(parse_ufloat::<f32>(" 13.37.hello "), Some(13.37f32));
    assert_eq!(parse_ufloat::<f32>(" 13.37e2.hello "), Some(1337f32));
    assert_eq!(parse_ufloat::<f32>(" 13.37e-2.hello "), Some(0.1337f32));

    // The decimal point is optional
    assert_eq!(parse_ufloat::<f32>(" 123 "), Some(123f32));
    assert_eq!(parse_ufloat::<f32>(" 0 "), Some(0f32));

    // Unlike parse_float(), a leading '-' is rejected entirely.
    assert_eq!(parse_ufloat::<f32>(" -123.45 "), None);
    assert_eq!(parse_ufloat::<f32>(" -0.0 "), None);
}

#[test]
fn test_parse_ufloat_f64() {
    assert_eq!(parse_ufloat::<f64>(" 123.hello "), Some(123f64));
    assert_eq!(parse_ufloat::<f64>(" +123.hello "), Some(123f64));
    assert_eq!(parse_ufloat::<f64>(" 13.37.hello "), Some(13.37f64));
    assert_eq!(parse_ufloat::<f64>(" 13.37e2.hello "), Some(1337f64));
    assert_eq!(parse_ufloat::<f64>(" 13.37e-2.hello "), Some(0.1337f64));
    assert_eq!(
        parse_ufloat::<f64>(" 1337.0e-297f64 "),
        Some(1337.0e-297f64)
    );

    // The decimal point is optional
    assert_eq!(parse_ufloat::<f64>(" 123 "), Some(123f64));
    assert_eq!(parse_ufloat::<f64>(" 0 "), Some(0f64));

    // Unlike parse_float(), a leading '-' is rejected entirely.
    assert_eq!(parse_ufloat::<f64>(" -123.45 "), None);
    assert_eq!(parse_ufloat::<f64>(" -0.0 "), None);
}

#[test]
fn test_parse_float_edge_cases() {
    // Neither an integer nor a decimal part is present.
    assert_eq!(parse_float::<f64>("."), None);
    assert_eq!(parse_float::<f64>(""), None);
    assert_eq!(parse_float::<f64>("   "), None);
    assert_eq!(parse_float::<f64>("-."), None);
    assert_eq!(parse_float::<f64>("-.e10"), None);

    // An integer or a decimal part alone is enough.
    assert_eq!(parse_float::<f64>(".5"), Some(0.5f64));
    assert_eq!(parse_float::<f64>("5."), Some(5f64));
    assert_eq!(parse_float::<f64>("-.5"), Some(-0.5f64));

    // Leading '+' is accepted just like '-'.
    assert_eq!(parse_float::<f64>("+1.5"), Some(1.5f64));

    // Trailing garbage is ignored, just like leading whitespace.
    assert_eq!(parse_float::<f64>("1.5xyz"), Some(1.5f64));

    // Without whitespace skipping, leading whitespace is rejected.
    assert_eq!(
        parse_float_from_iter::<f64>(&mut " 1.5".chars().peekable(), false),
        None
    );
    assert_eq!(
        parse_float_from_iter::<f64>(&mut "1.5".chars().peekable(), false),
        Some(1.5f64)
    );
}

#[test]
fn test_readme() {
    assert_eq!(parse_ufloat::<f32>("+123.45 as f32 "), Some(123.45f32));
    assert_eq!(parse_float::<f32>(" -123.45 as f32 "), Some(-123.45f32));
    assert_eq!(parse_ufloat::<f64>("+123.45 as f64 "), Some(123.45f64));
    assert_eq!(parse_float::<f64>(" -123.45 as f64 "), Some(-123.45f64));
    assert_eq!(parse_ufloat::<f32>("0"), Some(0f32));
    assert_eq!(parse_float::<f64>(" 123 as f64 "), Some(123f64));

    assert_eq!(parse_float::<f64>(" - 1.0 is invalid "), None);
    assert_eq!(
        parse_ufloat::<f64>(" -123.45 as f64, parse_float() not available for this value "),
        None
    );
}
