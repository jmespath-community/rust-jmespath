use crate::function;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(abs, [ subject => Required(Of(DataType::Number)) ], |_: &abs, args: &Vec<Value>, _: &dyn FunctionContext| {
    let num = args[0].as_f64().unwrap().abs();
    Value::from_f64(num)
});

#[cfg(test)]
mod tests {
    use crate::errors::Error as RuntimeError;
    use crate::functions::Function;
    use crate::{FunctionContext, Runtime, Value};
    use rstest::*;

    struct Fixture {
        runtime: Runtime,
    }
    impl FunctionContext for Fixture {
        fn create_by_function<'a>(
            &'a self,
            _ast: &'a crate::AST,
            _params: &'a Vec<crate::functions::ParamTypes>,
            _function: &'a dyn Function,
            _param_index: usize,
        ) -> Result<crate::ByFunctionHolder<'a>, RuntimeError> {
            todo!()
        }
    }

    fn setup() -> Fixture {
        let runtime = Runtime::create_runtime();
        Fixture { runtime }
    }

    #[rstest]
    #[case(3.into(), Value::from_f64(-3.0).unwrap())]
    fn abs(#[case] expected: Value, #[case] input: Value) {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "abs";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
