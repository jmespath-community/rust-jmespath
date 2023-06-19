use crate::function;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(ceil, [ subject => Required(Of(DataType::Number)) ], |_: &ceil, args: &Vec<Value>, _: &dyn FunctionContext| {
    let num = args[0].as_f64().unwrap().ceil();
    Value::from_f64(num)
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::from_f64(2.0).unwrap(), Value::from_f64(1.9).unwrap())]
    #[case(Value::from_f64(2.0).unwrap(), Value::from_f64(1.001).unwrap())]
    #[case(Value::from_f64(1.0).unwrap(), Value::from_f64(1f64).unwrap())]
    fn ceil(#[case] expected: Value, #[case] input: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "ceil";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
