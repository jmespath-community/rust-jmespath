use crate::{errors::Kind, value_eq::float_eq, Error};

/// Represents a JSON [`f64`] number that can be safely ordered.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number {
    pub(crate) number: f64,
}
impl Number {
    /// Creates a new instance of the [`Number`] type.
    ///
    /// This only accepts numbers that can be safely compared,
    /// _i.e_  `number` values that do not satisfy the following
    /// condition:
    /// ```compile_fail
    /// number.is_nan() || number.is_infinite()
    pub fn from(number: f64) -> Result<Self, Error> {
        if number.is_nan() || number.is_infinite() {
            return Err(Error::new(
                Kind::NotANumber,
                "An invalid number was specified.",
            ));
        }
        Ok(Number { number })
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let number = if float_eq(self.number, self.number.floor()) {
            self.number.floor()
        } else {
            self.number
        };
        write!(f, "{}", number)
    }
}

macro_rules! eq {
    ($type:ty) => {
        impl PartialEq<$type> for Number {
            fn eq(&self, other: &$type) -> bool {
                self.number == *other as f64
            }
        }
        impl PartialEq<Number> for $type {
            fn eq(&self, other: &Number) -> bool {
                other == self
            }
        }
    };
}

eq!(i8);
eq!(i16);
eq!(i32);
eq!(i64);

eq!(u8);
eq!(u16);
eq!(u32);
eq!(u64);

eq!(isize);
eq!(usize);

eq!(f32);
eq!(f64);

macro_rules! from {
    ($type:ty) => {
        impl From<$type> for Number {
            fn from(value: $type) -> Self {
                Number {
                    number: value as f64,
                }
            }
        }
    };
}

from!(i8);
from!(i16);
from!(i32);
from!(i64);

from!(u8);
from!(u16);
from!(u32);
from!(u64);

from!(isize);
from!(usize);

impl From<Number> for f64 {
    fn from(value: Number) -> Self {
        value.number
    }
}
impl From<&Number> for f64 {
    fn from(value: &Number) -> Self {
        value.number
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Number {}
impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(self.number.is_finite());
        assert!(!self.number.is_nan());
        self.number.partial_cmp(&other.number).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::cmp::Ordering::{self, *};

    #[rstest]
    #[case(f64::NAN)]
    #[case(f64::INFINITY)]
    #[case(f64::NEG_INFINITY)]
    fn ctor(#[case] input: f64) {
        assert!(Number::from(input).is_err());
    }

    #[rstest]
    #[case("42", Number::from(42.0).unwrap())]
    #[case("42.1", Number::from(42.10).unwrap())]
    fn it_implements_display_trait(#[case] expected: &str, #[case] number: Number) {
        assert_eq!(expected, format!("{}", number));
    }

    #[rstest]
    #[case(true, Number::from(42.0).unwrap(), Number::from(42.0).unwrap())]
    #[case(false, Number::from(42.0).unwrap(), Number::from(42.01).unwrap())]
    fn it_implements_eq_trait(#[case] expected: bool, #[case] left: Number, #[case] right: Number) {
        assert_eq!(expected, left == right);
    }

    macro_rules! eq {
        ($ident:ident, $expr:expr) => {
            #[test]
            fn $ident() {
                assert!($expr == Number::from($expr as f64).unwrap());
                assert!(Number::from($expr as f64).unwrap() == $expr);
            }
        };
    }
    eq!(if_implements_partial_eq_i8, 42i8);
    eq!(if_implements_partial_eq_i16, 42i16);
    eq!(if_implements_partial_eq_i32, 42i32);
    eq!(if_implements_partial_eq_i64, 42i64);

    eq!(if_implements_partial_eq_u8, 42u8);
    eq!(if_implements_partial_eq_u16, 42u16);
    eq!(if_implements_partial_eq_u32, 42u32);
    eq!(if_implements_partial_eq_u64, 42u64);

    eq!(if_implements_partial_eq_isize, 42isize);
    eq!(if_implements_partial_eq_usize, 42usize);

    eq!(if_implements_partial_eq_f32, 42.0f32);
    eq!(if_implements_partial_eq_f64, 42.0);

    macro_rules! from {
        ($ident:ident, $expr:expr) => {
            #[test]
            fn $ident() {
                let number: Number = $expr.into();
                assert_eq!(number, $expr);
            }
        };
    }

    from!(from_i8, 42i8);
    from!(from_i16, 42i16);
    from!(from_i32, 42i32);
    from!(from_i64, 42i64);

    from!(from_u8, 42u8);
    from!(from_u16, 42u16);
    from!(from_u32, 42u32);
    from!(from_u64, 42u64);

    from!(from_isize, 42isize);
    from!(from_usize, 42usize);

    #[test]
    fn it_implements_to_f64() {
        let num: f64 = Number::from(42.0).unwrap().into();
        assert_eq!(42.0, num);
    }

    #[rstest]
    #[case(Less, Number::from(42.0).unwrap(), Number::from(43.0).unwrap())]
    #[case(Equal, Number::from(42.0).unwrap(), Number::from(42.0).unwrap())]
    #[case(Greater, Number::from(42.0).unwrap(), Number::from(41.0).unwrap())]
    fn it_implements_partial_ord(
        #[case] expected: Ordering,
        #[case] left: Number,
        #[case] right: Number,
    ) {
        assert_eq!(Some(expected), left.partial_cmp(&right))
    }
}
