use crate::functions::DataType;
use crate::parser::AST;
use crate::utils::Number;
use crate::{Error, Map};

/// Represents any valid value that is processed during evaluation
/// of a JMESPath expression or used as an argument to a JMESPath
/// [Function](crate::functions::Function).
///
/// Values are specified at runtime when calling the function.
/// They must match the function's signature.
///
#[derive(Debug, Clone)]
pub enum Value {
    /// Represents a valid JSON array.
    /// # Example
    /// ```
    /// use jmespath::Value;
    /// let value: Value = vec![1, 2, 3].into();
    /// assert!(matches!(value, Value::Array(..)));
    Array(Vec<Value>),
    /// Represents a valid JSON boolean.
    /// # Example
    /// ```
    /// use jmespath::Value;
    /// let value: Value = true.into();
    /// assert!(matches!(value, Value::Boolean(true)));
    Boolean(bool),
    /// Represents a valid JSON null token.
    /// # Example
    /// ```
    /// use jmespath::Value;
    /// let value: Value = None.into();
    /// assert!(matches!(value, Value::Null));
    Null,
    /// Represents a valid JSON number.
    /// # Example
    /// ```
    /// use jmespath::Value;
    /// use jmespath::Number;
    /// let value: Value = 42.into();
    /// assert!(matches!(value, Value::Number(..)));
    Number(Number),
    /// Represents a valid JSON string.
    /// # Example
    /// ```
    /// use jmespath::Value;
    /// let value: Value = "text".into();
    /// assert!(matches!(value, Value::String(..)));
    String(String),
    /// Represents a valid JSON object.
    /// # Example
    /// ```
    /// use jmespath::{Map, Value};
    /// use jmespath::map;
    /// let value: Value = map!("foo" => "bar").into();
    /// assert!(matches!(value, Value::Object(..)));
    Object(Map<String, Value>),

    /// Represents a JMESPath expression.
    Expression(AST),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Expression(ast) => format!("{:?}", ast),
            _ => serde_json::to_string(self).map_err(|_| std::fmt::Error)?,
        };
        write!(f, "{}", text)
    }
}

macro_rules! as_ {
    ($ident:ident, $enum:ident, $type:ty) => {
        #[doc = "Returns an"]
        #[doc = concat!("[`Option<", stringify!($type), ">`]") ]
        #[doc = "of the corresponding type."]
        pub fn $ident(&self) -> Option<$type> {
            if let Self::$enum(token) = self {
                Some(token)
            } else {
                None
            }
        }
    };
}
macro_rules! as_deref {
    ($ident:ident, $enum:ident, $type:ty) => {
        #[doc = "Returns an"]
        #[doc = concat!("[`Option<", stringify!($type), ">`]") ]
        #[doc = "of the corresponding type."]
        pub fn $ident(&self) -> Option<$type> {
            if let Self::$enum(token) = self {
                Some(*token)
            } else {
                None
            }
        }
    };
}
macro_rules! if_ {
    ($ident:ident, $enum:ident) => {
        #[doc = "Returns [`Option<Value>`] if the [`Value`] is "]
        #[doc = stringify!($ident)]
        #[doc = "."]
        pub fn $ident(&self) -> Option<Value> {
            if matches!(self, Value::$enum(..)) {
                Some(self.clone())
            } else {
                None
            }
        }
    };
}
macro_rules! is_ {
    ($ident:ident, $type:ident) => {
        #[doc = "Returns `true` if the [`Value`] is a value from type"]
        #[doc = concat!("[`Value::", stringify!($type), "`].")]
        pub fn $ident(&self) -> bool {
            matches!(*self, Self::$type(..))
        }
    };
}

impl Value {
    pub fn from_f64(number: f64) -> Result<Self, Error> {
        match Number::from(number) {
            Err(err) => Err(err),
            Ok(n) => Ok(Value::Number(n)),
        }
    }
    /// Converts a [`serde_json::Value`] to a [`Value`].
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use jmespath::Value;
    ///
    /// let s = json!({"foo": "bar"});
    /// let v = Value::map_from_json(&s);
    /// assert!(matches!(v, Value::Object(..)));
    pub fn map_from_json(value: &serde_json::Value) -> Value {
        match value {
            serde_json::Value::Array(a) => {
                Value::Array(a.into_iter().map(|x| Self::map_from_json(x)).collect())
            }
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Number(n) => {
                let num = Number::from(n.as_f64().unwrap()).unwrap();
                Value::Number(num)
            }
            serde_json::Value::String(s) => Value::String(s.to_string()),
            serde_json::Value::Object(m) => {
                let map: Map<String, Value> = m
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::map_from_json(v)))
                    .collect();
                Value::Object(map)
            }
        }
    }
    /// Creates a [`Value`] from a JSON representation.
    ///
    /// Convenience function that uses `serde_json` to convert
    /// a JSON representation to a [`Value`].
    ///
    /// # Example
    ///
    /// ```
    /// use jmespath::Value;
    ///
    /// let value = Value::from_json(r#"{"foo": "bar"}"#).unwrap();
    /// assert!(value.is_object());
    /// ```
    ///
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        match text {
            "null" => Ok(Value::Null),
            _ => {
                let obj: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(text);
                match obj {
                    Err(err) => Err(err),
                    Ok(v) => Ok(Self::map_from_json(&v)),
                }
            }
        }
    }
    /// Returns the JSON string representation
    /// for this [`Value`].
    ///
    /// # Example
    ///
    /// ```
    /// use jmespath::Value;
    ///
    /// assert_eq!("null", Value::Null.to_json());
    /// assert_eq!("\"text\"", Value::String("text".to_string()).to_json());
    /// ```
    pub fn to_json(&self) -> String {
        // safe to expect as serde_json::to_string() can fail only
        // if Value's implementation of Serialize decides to fail,
        // or if Value contains a map with non-string keys.
        serde_json::to_string(self).expect("unable to serialize invalid JSON")
    }

    /// Returns the corresponding [`DataType`]
    ///
    /// # Example
    ///
    /// ```
    /// use jmespath::Value;
    /// use jmespath::functions::DataType;
    ///
    /// let arg = Value::from_f64(42.0).unwrap();
    /// assert_eq!(DataType::Number, arg.get_data_type());
    /// ```
    ///
    pub fn get_data_type(&self) -> DataType {
        match self {
            Self::Array(_) => DataType::Array,
            Self::Boolean(_) => DataType::Boolean,
            Self::Null => DataType::Null,
            Self::Number(_) => DataType::Number,
            Self::Object(_) => DataType::Object,
            Self::String(_) => DataType::String,
            Self::Expression(_) => DataType::ExpRef,
        }
    }

    /// Converts a [`Vec<T>`] to a [`Vec<Value>`]
    /// # Example
    ///
    /// ```
    /// use jmespath::Value;
    /// let vec: Vec<Value> = Value::map_into(vec![1, 2]);
    ///  assert!(vec.iter().all(|x| matches!(x, Value::Number(..))));
    #[inline]
    pub fn map_into<T>(vec: Vec<T>) -> Vec<Self>
    where
        T: Into<Value>,
    {
        vec.into_iter().map(|x| x.into()).collect::<Vec<Value>>()
    }

    //
    //    /// Casts the [`Value`] to [`Option<()`].
    //    pub fn as_null(&self) -> Option<()> {
    //        if self.is_null() {
    //            Some(())
    //        } else {
    //            None
    //        }
    //    }
    //
    as_!(as_array, Array, &Vec<Value>);
    as_!(as_expref, Expression, &AST);
    as_!(as_number, Number, &Number);
    as_!(as_object, Object, &Map<String, Value>);
    as_!(as_str, String, &str);

    as_deref!(as_bool, Boolean, bool);

    /// Returns an [`Option<f64>`] of the corresponding type.
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Number(Number { number: token }) = self {
            Some(*token)
        } else {
            None
        }
    }

    if_!(if_array, Array);
    if_!(if_object, Object);
    //
    //    /// Applies a reduce function to the [`Value`] if it is an array, otherwise, returns None.
    //    pub fn reduce_if_array<F>(&self, acc: Value, f: F) -> Option<Value>
    //    where
    //        F: Fn(&mut Value, &Value) -> Value,
    //    {
    //        match self {
    //            Value::Array(array) => {
    //                let mut acc: Value = acc.clone();
    //                for item in array {
    //                    acc = f(&mut acc, &item)
    //                }
    //                Some(acc)
    //            }
    //            _ => None,
    //        }
    //    }
    //
    //    /// Returns [`Option<Value>`] if it is the `null` value.
    //    pub fn if_null(&self) -> Option<Value> {
    //        if self.is_null() {
    //            Some(Value::Null)
    //        } else {
    //            None
    //        }
    //    }
    //

    /// Returns `true` if the [`Value`] is the `null` value.
    pub fn is_null(&self) -> bool {
        matches!(*self, Self::Null)
    }

    is_!(is_array, Array);
    is_!(is_bool, Boolean);
    is_!(is_number, Number);
    is_!(is_object, Object);
    is_!(is_str, String);

    is_!(is_expression, Expression);

    /// Returns `true` if the [`Value`] is the boolean `true`.
    pub fn is_true(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }
    /// Returns `false` if the [`Value`] is the boolean `false`.
    pub fn is_false(&self) -> bool {
        matches!(self, Value::Boolean(false))
    }

    /// Returns `true` if the [`Value`] is either:
    /// - the `null` value
    /// - the boolean `false`
    /// - the empty string `""`
    /// - an empty array `[]`
    /// - an empty object `{}`
    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Array(a) => a.is_empty(),
            Value::Boolean(b) => !*b,
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Object(o) => o.is_empty(),
            _ => false,
        }
    }
    /// Returns `true` if the [`Value`] is not a _falsy_ value.
    /// This is the opposite to the [`Value::is_falsy()`] function.
    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors::Position, map, NodeType};
    use rstest::*;

    #[rstest]
    #[case("[1.0,2.0,3.0]", Value::Array(vec![Value::Number(1i8.into()), Value::Number(2i8.into()), Value::Number(3i8.into())]))]
    #[case("true", Value::Boolean(true))]
    #[case("false", Value::Boolean(false))]
    #[case("null", Value::Null)]
    #[case("42.0", Value::Number(42i8.into()))]
    #[case(r#"{"foo":"bar"}"#, Value::Object(map!("foo".to_string() => Value::String("bar".to_string()))))]
    #[case(r#""foo""#, Value::String("foo".to_string()))]
    fn it_implements_display_trait(#[case] expected: &str, #[case] value: Value) {
        assert_eq!(expected, format!("{}", value));
    }

    #[rstest]
    #[case(vec![1, 2, 3].into(), DataType::Array)]
    #[case(true.into(), DataType::Boolean)]
    #[case(42i8.into(), DataType::Number)]
    #[case("foo".into(), DataType::String)]
    #[case(None.into(), DataType::Null)]
    #[case(map!("foo" => "bar").into(), DataType::Object)]
    fn it_maps_to_a_data_type(#[case] arg: Value, #[case] expected_data_type: DataType) {
        assert_eq!(expected_data_type, arg.get_data_type());
    }

    #[test]
    fn it_maps_to_expref() {
        let current = AST::make(NodeType::CurrentNode, Position::new(1, 2));
        let expref = AST::make(NodeType::Expression(vec![current]), Position::new(1, 1));
        let value = Value::Expression(expref);
        assert_eq!(DataType::ExpRef, value.get_data_type());
    }

    #[rstest]
    #[case(None.into(), "null")]
    #[case(true.into(), "true")]
    #[case(false.into(), "false")]
    #[case("text".into(), "\"text\"")]
    #[case(42.into(), "42.0")]
    #[case(map!("text" => None).into(), r#"{"text": null}"#)]
    #[case(map!("foo" => map!("bar" => "baz")).into(), r#"{"foo": {"bar": "baz"}}"#)]
    fn from_json(#[case] expected: Value, #[case] json: &str) {
        assert_eq!(expected, Value::from_json(json).unwrap());
    }

    #[test]
    fn from_json_err() {
        assert!(Value::from_json("{").is_err())
    }

    #[test]
    fn from_f64() {
        assert!(Value::from_f64(42.0).is_ok())
    }

    #[rstest]
    #[case("[1.0,2.0,3.0]", vec![1i8, 2i8, 3i8].into())]
    #[case("true", true.into())]
    #[case("false", false.into())]
    #[case("null", Value::Null)]
    #[case("1.0", 1i8.into())]
    #[case(r#"{"foo":"bar"}"#, map!("foo" => "bar").into())]
    #[case("\"text\"", "text".into())]
    fn to_json(#[case] expected: &str, #[case] input: Value) {
        assert_eq!(expected, input.to_json());
    }

    #[test]
    fn value_as() {
        assert!(Value::Null.as_f64().is_none());
    }

    #[test]
    fn if_array() {
        assert!(matches!(Value::Array(vec![]).if_array(), Some(..)));
    }
    #[test]
    fn if_object() {
        assert!(matches!(Value::Object(Map::new()).if_object(), Some(..)));
    }

    #[test]
    fn is_array() {
        assert_eq!(true, Value::Array(vec![]).is_array())
    }

    #[test]
    fn is_boolean() {
        assert_eq!(true, Value::Boolean(true).is_bool());
        assert_eq!(false, Value::Null.is_bool());
    }
    #[test]
    fn is_null() {
        assert_eq!(true, Value::Null.is_null());
        assert_eq!(false, Value::Boolean(false).is_null());
    }
    #[test]
    fn is_number() {
        assert_eq!(true, Value::from_f64(1.0).unwrap().is_number())
    }

    #[test]
    fn is_str() {
        assert_eq!(true, Value::String("text".to_string()).is_str())
    }

    #[test]
    fn is_object() {
        assert_eq!(true, Value::Object(Map::new()).is_object())
    }

    #[rstest]
    #[case(Value::Null, true)]
    #[case(Value::Boolean(false), true)]
    #[case(Value::String("".to_string()), true)]
    #[case(Value::Object(map!()), true)]
    #[case(Value::Array([].into()), true)]
    #[case(Value::Number(42.into()), false)]
    fn falsy(#[case] value: Value, #[case] expected: bool) {
        assert_eq!(expected, value.is_falsy());
        assert_eq!(!value.is_truthy(), value.is_falsy());
    }
}
