use crate::function;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(length, [ subject => Required(Any(vec![DataType::Array, DataType::Object, DataType::String])) ], |_: &length, args: &Vec<Value>, _: &dyn FunctionContext| {
    let length = match &args[0] {
        Value::Array(v) => v.len(),
        Value::Object(o) => o.len(),
        Value::String(s) => s.chars().count(),
        _ => unreachable!(),
    };
    Ok(length)
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error as RuntimeError;
    use crate::map;
    use crate::Map;
    use crate::Runtime;
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
    #[case(Value::from_f64(3.0).unwrap(), "foo".into())]
    #[case(Value::from_f64(3.0).unwrap(), map!("foo" => "foo", "bar" => "bar", "baz" => "baz").into())]
    #[case(Value::from_f64(3.0).unwrap(), vec!["foo", "bar", "baz"].into())]
    fn length(#[case] expected: Value, #[case] input: Value) {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "length";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert!(result.is_number());
        assert_eq!(expected, result);
    }
}
