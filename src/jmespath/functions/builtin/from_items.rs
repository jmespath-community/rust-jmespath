use crate::errors::error_builder::ErrorBuilder;
use crate::errors::error_builder::FunctionErrorBuilder;
use crate::errors::error_builder::InvalidValueErrorBuilder;
use crate::errors::invalid_value::InvalidValueErrorBuilderFactory;
use crate::errors::Error as RuntimeError;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Map;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

use crate::function;

function!(from_items, [ subject => Required(Of(DataType::Array)) ], |me: &from_items, args: &Vec<Value>, _: &dyn FunctionContext| {
    let array = args[0].as_array().unwrap();

    let mut map: Map<String, Value> = Map::new();

    for index in 0..array.len() {
        match array[index].as_array() {
            Some(item) => {
                match item.as_slice() {
                    [Value::String(key), value] => map.insert(key.to_string(), value.clone()),
                    _ => return Err(me.build_invalid_value_error("subject", &args[0])),
                };
            }
            _ => return Err(me.build_invalid_value_error("subject", &args[0])),
        }
    }

    Ok(Value::Object(map))
});

impl from_items {
    fn build_invalid_value_error(&self, parameter: &str, received: &Value) -> RuntimeError {
        RuntimeError::get_invalid_value_error_builder()
        .for_function(self.get_name())
        .for_parameter(parameter)
        .received(received)
        .expected("an array whose elements are each an array of two elements, a pair of string and value")
        .build()
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::Kind;
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::from_json(r#"{"one":1.0,"two":2.0}"#).unwrap(), Value::from_json(r#"[["one", 1], ["two", 2]]"#).unwrap())]
    #[case(Value::from_json(r#"{"one":3.0,"two":2.0}"#).unwrap(), Value::from_json(r#"[["one", 1], ["two", 2], ["one", 3]]"#).unwrap())]
    fn from_items(#[case] expected: Value, #[case] input: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "from_items";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(Kind::InvalidValue, Value::from_json(r#"[["one"], ["two", 2]]"#).unwrap())]
    #[case(Kind::InvalidValue, Value::from_json(r#"[[1], ["two", 2]]"#).unwrap())]
    fn from_items_err(#[case] expected: Kind, #[case] input: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "from_items";
        let args = vec![input];
        let result = fixture
            .runtime
            .call(fname, &args, context)
            .map_err(|x| x.kind);

        if let Err(kind) = result {
            assert_eq!(expected, kind)
        } else {
            assert!(false)
        }
    }
}
