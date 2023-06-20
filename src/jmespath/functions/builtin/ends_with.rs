use crate::function;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(ends_with, [
    subject => Required(Of(DataType::String)),
    suffix => Required(Of(DataType::String))
    ], |_: &ends_with, args: &Vec<Value>, _: &dyn FunctionContext| {
        let subject: Vec<_> = args[0].as_str().unwrap().chars().collect();
        let suffix: Vec<_> = args[1].as_str().unwrap().chars().collect();
        let ends_with = subject.ends_with(&suffix);
        Ok(Value::Boolean(ends_with))
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::Boolean(true), Value::String("substring".to_string()), Value::String("string".to_string()))]
    #[case(Value::Boolean(false), Value::String("substring".to_string()), Value::String("sub".to_string()))]
    fn ends_with(#[case] expected: Value, #[case] subject: Value, #[case] suffix: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "ends_with";
        let args = vec![subject, suffix];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
