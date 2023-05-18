use crate::errors::Error;
use crate::runtime::Runtime;
use crate::Value;

/// Evaluates a JMESPath expression and returns a [`Value`].
///
/// # Example
///
/// ```
/// use jmespath::search;
/// use jmespath::map;
/// use jmespath::Map;
/// use serde_json;
///
/// let input = map!("foo"=> "bar").into();
/// let expression = "'foo'";
/// let result = search(expression, &input).unwrap();
///
/// assert_eq!("\"foo\"", serde_json::to_string(&result).unwrap());
/// ```
pub fn search(expression: &str, root: &Value) -> Result<Value, Error> {
    Runtime::get_shared_runtime().search(expression, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_search() {
        let root = Value::from_json(r#"{"foo": "bar"}"#).unwrap();
        let result = search("foo", &root).unwrap();
        assert_eq!("\"bar\"", result.to_json());
    }
}
