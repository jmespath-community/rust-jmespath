use crate::{Value, utils::Number};

impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Expression(_) => false,
            Value::Null => other.is_null(),
            Value::Array(a) => other.is_array() && Some(a) == other.as_array(),
            Value::Boolean(b) => other.is_bool() && Some(*b) == other.as_bool(),
            Value::Number(Number { number: n }) => {
                other.is_number() && float_eq(*n, other.as_number().unwrap().into())
            }
            Value::String(s) => other.is_str() && s == other.as_str().unwrap(),
            Value::Object(o) => other.is_object() && Some(o) == other.as_object(),
        }
    }
}

impl PartialEq<Value> for bool {
    fn eq(&self, other: &Value) -> bool {
        other == self
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        self.is_bool()
            && if *other {
                self.is_true()
            } else {
                self.is_false()
            }
    }
}

impl PartialEq<Value> for Option<()> {
    fn eq(&self, other: &Value) -> bool {
        other == self
    }
}
impl PartialEq<Option<()>> for Value {
    fn eq(&self, _: &Option<()>) -> bool {
        self.is_null()
    }
}

impl PartialEq<Value> for &str {
    fn eq(&self, other: &Value) -> bool {
        other == self
    }
}
impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        self.is_str() && other == &self.as_str().unwrap()
    }
}
impl PartialEq<Value> for String {
    fn eq(&self, other: &Value) -> bool {
        other == self
    }
}
impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        self.is_str() && other == &self.as_str().unwrap()
    }
}

macro_rules! eq_num_lhs {
    ($int_type:ty) => {
        impl PartialEq<$int_type> for Value {
            fn eq(&self, other: &$int_type) -> bool {
                self.is_number() && float_eq(self.as_f64().unwrap(), (*other) as f64)
            }
        }
    };
}
eq_num_lhs!(i8);
eq_num_lhs!(i16);
eq_num_lhs!(i32);
eq_num_lhs!(i64);

eq_num_lhs!(u8);
eq_num_lhs!(u16);
eq_num_lhs!(u32);
eq_num_lhs!(u64);

eq_num_lhs!(f32);
eq_num_lhs!(f64);

eq_num_lhs!(isize);
eq_num_lhs!(usize);

macro_rules! eq_num_rhs {
    ($int_type:ty) => {
        impl PartialEq<Value> for $int_type {
            fn eq(&self, other: &Value) -> bool {
                other == self
            }
        }
    };
}

eq_num_rhs!(i8);
eq_num_rhs!(i16);
eq_num_rhs!(i32);
eq_num_rhs!(i64);

eq_num_rhs!(u8);
eq_num_rhs!(u16);
eq_num_rhs!(u32);
eq_num_rhs!(u64);

eq_num_rhs!(f32);
eq_num_rhs!(f64);

eq_num_rhs!(isize);
eq_num_rhs!(usize);

/// Compares two floating point numbers
#[inline]
pub(crate) fn float_eq(a: f64, b: f64) -> bool {
    let diff = (b - a).abs();
    return diff < std::f64::EPSILON;
}

#[cfg(test)]
mod tests {
    use crate::map;
    use crate::utils::Number;
    use crate::{Map, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::Boolean(true), true.into(), true)]
    #[case(Value::Number(Number::from(42.000000000000000001).unwrap()), 42.into(), true)]
    #[case(Value::String("str".to_string()), "str".into(), true)]
    #[case(Value::Null, None.into(), true)]
    #[case(Value::from_json(r#"{"foo": "bar"}"#).unwrap(), map!("foo" => "bar").into(), true)]
    #[case(Value::from_json(r#"["one", 2, {"three": 3}]"#).unwrap(), vec!["one".into(), <i32 as Into<Value>>::into(2), map!("three" => 3).into()].into(), true)]
    fn it_implements_partial_eq(#[case] left: Value, #[case] right: Value, #[case] expected: bool) {
        assert_eq!(expected, left == right);
    }

    #[test]
    fn it_implements_partial_eq_bool_lhs() {
        assert_eq!(Value::Boolean(true), true);
        assert_eq!(Value::Boolean(false), false);
    }
    #[test]
    fn it_implements_partial_eq_bool_rhs() {
        assert_eq!(true, Value::Boolean(true));
        assert_eq!(false, Value::Boolean(false));
    }

    #[test]
    fn it_implements_partial_eq_null_lhs() {
        assert_eq!(Value::Null, None);
        assert_eq!(Value::Null, Some(()));
    }
    #[test]
    fn it_implements_partial_eq_null_rhs() {
        assert_eq!(None, Value::Null);
        assert_eq!(Some(()), Value::Null);
    }

    macro_rules! eq_num_lhs {
        ($ident:ident, $type:ty, $expected:expr, $input:expr) => {
            #[test]
            fn $ident() {
                assert_eq!(
                    $expected,
                    Value::Number(Number::from($input as f64).unwrap())
                );
            }
        };
    }

    eq_num_lhs!(it_implements_partial_eq_i8_lhs, i8, 42.0, 42i8);
    eq_num_lhs!(it_implements_partial_eq_i16_lhs, i16, 42.0, 42i16);
    eq_num_lhs!(it_implements_partial_eq_i32_lhs, i32, 42.0, 42i32);
    eq_num_lhs!(it_implements_partial_eq_u8_lhs, u8, 42.0, 42u8);
    eq_num_lhs!(it_implements_partial_eq_u16_lhs, u16, 42.0, 42u16);
    eq_num_lhs!(it_implements_partial_eq_u32_lhs, u32, 42.0, 42u32);

    eq_num_lhs!(it_implements_partial_eq_isize_lhs, isize, 42.0, 42isize);
    eq_num_lhs!(it_implements_partial_eq_usize_lhs, usize, 42.0, 42usize);

    eq_num_lhs!(it_implements_partial_eq_f32_lhs, f32, 42.0, 42.0f32);
    eq_num_lhs!(it_implements_partial_eq_f64_lhs, f64, 42.0, 42.0f64);

    macro_rules! eq_num_rhs {
        ($ident:ident, $type:ty, $expected:expr, $input:expr) => {
            #[test]
            fn $ident() {
                assert_eq!(
                    $expected,
                    Value::Number(Number::from($input as f64).unwrap())
                );
            }
        };
    }
    eq_num_rhs!(it_implements_partial_eq_i8_rhs, i8, 42.0, 42i8);
    eq_num_rhs!(it_implements_partial_eq_i16_rhs, i16, 42.0, 42i16);
    eq_num_rhs!(it_implements_partial_eq_i32_rhs, i32, 42.0, 42i32);
    eq_num_rhs!(it_implements_partial_eq_u8_rhs, u8, 42.0, 42u8);
    eq_num_rhs!(it_implements_partial_eq_u16_rhs, u16, 42.0, 42u16);
    eq_num_rhs!(it_implements_partial_eq_u32_rhs, u32, 42.0, 42u32);

    eq_num_rhs!(it_implements_partial_eq_isize_rhs, isize, 42.0, 42isize);
    eq_num_rhs!(it_implements_partial_eq_usize_rhs, usize, 42.0, 42usize);

    eq_num_rhs!(it_implements_partial_eq_f32_rhs, f32, 42.0, 42.0f32);
    eq_num_rhs!(it_implements_partial_eq_f64_rhs, f64, 42.0, 42.0f64);

    #[test]
    fn it_implements_partial_eq_str_lhs() {
        assert_eq!(Value::String("text".to_string()), "text");
    }
    #[test]
    fn it_implements_partial_eq_str_rhs() {
        assert_eq!("text", Value::String("text".to_string()));
    }
    #[test]
    fn it_implements_partial_eq_string_lhs() {
        let text = "text".to_string();
        assert_eq!(Value::String("text".to_string()), text);
    }
    #[test]
    fn it_implements_partial_eq_string_rhs() {
        let text = "text".to_string();
        assert_eq!(text, Value::String("text".to_string()));
    }

    #[rstest]
    #[case(true, 42.0, 42.0)]
    #[case(false, 42.0, 43.0)]
    #[case(true, 0.0, std::f64::EPSILON/2.0)]
    #[case(false, 0.0, std::f64::EPSILON)]
    fn float_eq(#[case] expected: bool, #[case] left: f64, #[case] right: f64) {
        assert_eq!(expected, super::float_eq(left, right));
    }
}
