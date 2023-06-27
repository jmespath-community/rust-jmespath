use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

use crate::function;

function!(items, [ obj => Required(Of(DataType::Object)) ], |_: &items, args: &Vec<Value>, _: &dyn FunctionContext| {
    let obj = args[0].as_object().unwrap();
    let array: Vec<_> = obj.iter().map(|item| Value::Array(vec![Value::String(item.0.to_string()), item.1.clone()])).collect();

    Ok(Value::Array(array))
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::from_json(r#"[["one",1.0],["two",2.0]]"#).unwrap(), Value::from_json(r#"{"one": 1, "two": 2}"#).unwrap())]
    #[case(Value::from_json(r#"[["one",3.0],["two",2.0]]"#).unwrap(), Value::from_json(r#"{"one": 3, "two": 2}"#).unwrap())]
    fn items(#[case] expected: Value, #[case] input: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "items";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
