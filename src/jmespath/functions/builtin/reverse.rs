use crate::function;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(reverse, [ subject => Required(Any(vec![DataType::Array, DataType::String])) ], |_: &reverse, args: &Vec<Value>, _: &dyn FunctionContext| {
    let reversed = match &args[0] {
        Value::Array(v) => {
            let mut vec = v.clone();
            vec.reverse();
            Value::Array(vec)
        },
        Value::String(s) => {
            let mut vec: Vec<char> = s.chars().collect();
            vec.reverse();
            let s: String = vec.into_iter().collect();
            Value::String(s)
        },
        _ => unreachable!(),
    };
    Ok(reversed)
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error as RuntimeError;
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
    #[case("oof".into(), "foo".into())]
    #[case(vec!["baz", "bar", "foo"].into(), vec!["foo", "bar", "baz"].into())]
    fn revserse(#[case] expected: Value, #[case] input: Value) {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "reverse";
        let args = vec![input];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
