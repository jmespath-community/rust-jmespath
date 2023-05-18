/// Represents a valid type for a [`crate::Value`] to a JMESPath Function.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DataType {
    Any,
    Array,
    Boolean,
    ExpRef,
    Null,
    Number,
    Object,
    String,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            DataType::Any => "any",
            DataType::Array => "array[any]",
            DataType::ExpRef => "expression",
            DataType::Object => "object",
            DataType::Boolean => "boolean",
            DataType::Null => "null",
            DataType::Number => "number",
            DataType::String => "string",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("any", DataType::Any)]
    #[case("array[any]", DataType::Array)]
    #[case("boolean", DataType::Boolean)]
    #[case("expression", DataType::ExpRef)]
    #[case("number", DataType::Number)]
    #[case("null", DataType::Null)]
    #[case("object", DataType::Object)]
    #[case("string", DataType::String)]
    fn it_formats_data_type(#[case] expected: &str, #[case] data_type: DataType) {
        assert_eq!(expected, format!("{}", data_type))
    }
}
