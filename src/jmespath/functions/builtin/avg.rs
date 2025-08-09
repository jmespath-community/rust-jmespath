use crate::function;

use crate::FunctionContext;
use crate::Value;
use crate::functions::ReturnValue;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(avg, [ subject => Required(Of(DataType::Array)) ], |_: &avg, args: &Vec<Value>, _: &dyn FunctionContext| {
    let array = args[0].as_array().unwrap();
    let numbers = array.iter().filter_map(|x| x.as_f64()).collect::<Vec<f64>>();

    if numbers.len() == 0 || numbers.len() != array.len() {
        Ok(Value::Null)
    }
    else {
        let sum: f64 = numbers.iter().sum();
        let count = numbers.len() as f64;

        Value::from_f64(sum / count)
    }
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(2.into(), vec![1, 2, 3].into())]
    #[case(None.into(), Value::Array(vec![]))]
    fn avg(#[case] expected: Value, #[case] input: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "avg";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
