use num;

/// JavaScript parseInt-style prefix parsing

pub fn parse_int<T: num::PrimInt + std::ops::Neg<Output = T>>(s: &str) -> T {
    let mut ret = T::zero();
    let mut neg = false;

    for (i, digit) in s.trim().chars().enumerate() {
        if i == 0 && (digit == '+' || digit == '-') {
            neg = digit == '-';
            continue;
        }

        match digit.to_digit(10) {
            Some(digit) => {
                ret = ret.checked_mul(&T::from(10).unwrap()).unwrap();
                ret = ret.checked_add(&T::from(digit).unwrap()).unwrap();
            }
            None => break,
        }
    }

    if neg { -ret } else { ret }
}

#[test]
fn test() {
    assert_eq!(parse_int::<i64>("123hello"), 123i64);
}
