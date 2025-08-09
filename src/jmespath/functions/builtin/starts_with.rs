use crate::function;

use crate::FunctionContext;
use crate::Value;
use crate::functions::ReturnValue;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(starts_with, [
    subject => Required(Of(DataType::String)),
    prefix => Required(Of(DataType::String))
    ], |_: &starts_with, args: &Vec<Value>, _: &dyn FunctionContext| {
        let subject: Vec<_> = args[0].as_str().unwrap().chars().collect();
        let prefix: Vec<_> = args[1].as_str().unwrap().chars().collect();
        let starts_with = subject.starts_with(&prefix);
        Ok(Value::Boolean(starts_with))
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::Boolean(false), Value::String("substring".to_string()), Value::String("string".to_string()))]
    #[case(Value::Boolean(true), Value::String("substring".to_string()), Value::String("sub".to_string()))]
    fn starts_with(#[case] expected: Value, #[case] subject: Value, #[case] prefix: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "starts_with";
        let args = vec![subject, prefix];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
