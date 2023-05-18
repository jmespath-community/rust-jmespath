use crate::{utils::Number, value::Value, Map};

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Array(a) => a.serialize(serializer),
            Self::Boolean(b) => serializer.serialize_bool(*b),
            Self::Null => serializer.serialize_unit(),
            Self::Number(n) => n.serialize(serializer),
            Self::Object(o) => o.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            _ => unreachable!(),
        }
    }
}
impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(self.number)
    }
}

#[cfg(test)]
mod tests {

    use crate::map;
    use crate::{Map, Value};
    use rstest::*;

    #[rstest]
    #[case(vec![1, 2, 3].into(), "[1.0,2.0,3.0]")]
    #[case(true.into(), "true")]
    #[case(false.into(), "false")]
    #[case(2.into(), "2.0")]
    #[case(None.into(), "null")]
    #[case(map!("foo"=>"bar").into(), "{\"foo\":\"bar\"}")]
    fn it_serializes_value_to_serde_json(#[case] arg: Value, #[case] expected: &str) {
        assert_eq!(serde_json::to_string(&arg).unwrap(), expected);
    }

    #[rstest]
    #[case(42i8.into(), "42.0")]
    fn it_serializes_number_to_serde_json(#[case] arg: Value, #[case] expected: &str) {
        assert_eq!(serde_json::to_string(&arg).unwrap(), expected);
    }

    #[test]
    fn it_serializes_heterogeneous_array() {
        let expected = "[[],true,false,null,1.0,{}]";
        let map: Map<&str, Value> = Map::new();
        let arg = vec![
            Value::Array(vec![]),
            true.into(),
            false.into(),
            None.into(),
            1.into(),
            map.into(),
        ];
        assert_eq!(serde_json::to_string(&arg).unwrap(), expected);
    }
}
