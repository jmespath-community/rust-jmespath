use crate::utils::Number;
use crate::{Map, Value};

impl<V> From<Map<&str, V>> for Value
where
    V: Into<Value>,
{
    fn from(v: Map<&str, V>) -> Self {
        let mut map: Map<String, Value> = Map::new();
        for (key, value) in v.into_iter() {
            map.insert(key.to_string(), value.into());
        }
        Self::Object(map)
    }
}
impl From<Number> for Value {
    fn from(number: Number) -> Self {
        Value::Number(number)
    }
}
impl From<Option<()>> for Value {
    fn from(_: Option<()>) -> Self {
        Self::Null
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Vec<T>) -> Self {
        let mut vec: Vec<Value> = Vec::new();
        for item in v {
            vec.push(item.into());
        }
        Self::Array(vec)
    }
}

macro_rules! from_ {
    ($ident:ty, $value:ident) => {
        impl From<$ident> for Value {
            fn from(v: $ident) -> Self {
                Self::$value(v.into())
            }
        }
    };
}
macro_rules! from_as {
    ($ident:ty, $as:ty, $value:ident) => {
        impl From<$ident> for Value {
            fn from(v: $ident) -> Self {
                Self::$value(Number::from((v as $as).into()).unwrap())
            }
        }
    };
}

from_! {bool, Boolean}
from_! {i8, Number}
from_! {i16, Number}
from_! {i32, Number}
from_! {u8, Number}
from_! {u16, Number}
from_! {u32, Number}

from_as! {isize, f64, Number}
from_as! {usize, f64, Number}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}
impl From<std::string::String> for Value {
    fn from(v: std::string::String) -> Self {
        Self::String(v.to_string())
    }
}
impl From<&std::string::String> for Value {
    fn from(v: &std::string::String) -> Self {
        Self::String(v.to_string())
    }
}

#[cfg(test)]
mod tests {

    use crate::map;
    use crate::utils::Number;
    use crate::{Map, Value};

    #[test]
    fn it_implements_from_map() {
        let map = map!("foo" => "bar");
        assert!(matches!(map.into(), Value::Object(..)));
    }
    #[test]
    fn it_implements_from_number() {
        let number = Number::from(42.0).unwrap();
        assert!(matches!(number.into(), Value::Number(..)));
    }

    #[test]
    fn it_implements_from_vec() {
        let vec = vec![1, 2];
        assert!(matches!(vec.into(), Value::Array(..)));
    }
    #[test]
    fn it_implements_from_vecref() {
        let text = "foo".to_string();
        let vec = vec![&text];
        assert!(matches!(vec.into(), Value::Array(..)));
    }
    #[test]
    fn it_implements_from_refvec() {
        let vec = vec![1, 2];
        assert!(matches!(&vec.into(), Value::Array(..)));
    }
    #[test]
    fn it_implements_from_refvecref() {
        let text = "foo".to_string();
        let vec = vec![&text];
        assert!(matches!(vec.into(), Value::Array(..)));
    }

    #[test]
    fn it_implements_from_bool() {
        assert!(matches!(true.into(), Value::Boolean(true)));
        assert!(matches!(false.into(), Value::Boolean(false)));
    }

    #[test]
    fn it_implements_from_null() {
        assert!(matches!(None.into(), Value::Null));
        assert!(matches!(Some(()).into(), Value::Null));
    }

    macro_rules! from_int {
        ($ident:ident, $expected:expr, $type:ty, $input:expr) => {
            #[test]
            fn $ident() {
                let value: Value = $input.into();
                assert_eq!($expected, value.as_f64().unwrap())
            }
        };
    }

    from_int!(it_implements_from_u8, 42.0, u8, 42u8);
    from_int!(it_implements_from_u16, 42.0, u16, 42u16);
    from_int!(it_implements_from_u32, 42.0, u32, 42u32);
    from_int!(it_implements_from_usize, 42.0, usize, 42usize);
    from_int!(it_implements_from_i8, 42.0, i8, 42i8);
    from_int!(it_implements_from_i16, 42.0, i16, 42i16);
    from_int!(it_implements_from_i32, 42.0, i32, 42i32);
    from_int!(it_implements_from_isize, 42.0, isize, 42isize);

    #[test]
    fn it_implements_from_refstr() {
        let str = "foo";
        assert!(matches!(str.into(), Value::String(..)));
    }
    #[test]
    fn it_implements_from_string() {
        let str = "foo".to_string();
        assert!(matches!(str.into(), Value::String(..)));
    }
    #[test]
    fn it_implements_from_refstring() {
        let str = "foo".to_string();
        assert!(matches!(&str.into(), Value::String(..)));
    }
}
