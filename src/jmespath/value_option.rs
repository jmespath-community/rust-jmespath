use crate::Value;

pub trait ValueOption {
    fn or_null(self) -> Value;
}
impl ValueOption for Option<Value> {
    fn or_null(self) -> Value {
        self.unwrap_or_else(|| Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use rstest::*;

    #[rstest]
    #[case(Value::Null, None)]
    #[case(Value::Boolean(true), Some(Value::Boolean(true)))]
    fn or_null(#[case] expected: Value, #[case] option: Option<Value>) {
        assert_eq!(expected, option.or_null())
    }
}
