use crate::errors::Error as RuntimeError;
use crate::function;
use crate::Number;

use crate::functions::ReturnValue;
use crate::FunctionContext;
use crate::Value;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(min_by, [
    elements => Required(Of(DataType::Array)),
    expr => Required(Of(DataType::ExpRef))
    ], |me: &min_by, args: &Vec<Value>, context: &dyn FunctionContext| {

        let array = args[0].as_array().unwrap();
        let ast = args[1].as_expref().unwrap();

        let params = vec![Of(DataType::Number)];
        let closure = context.create_by_function(&ast, &params, me, 1).unwrap();

        let numbers = array
            .iter()
            .map(|x| closure(x).map(|x| x.as_number().unwrap().clone()))
            .collect::<Result<Vec<Number>, RuntimeError>>()?;

        let tuple = array.iter().zip(numbers).min_by_key(|x| x.1);
        match tuple {
            Some((min, _)) => Ok(min.clone()),
            None => Ok(Value::Null),
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Kind;
    use crate::errors::Position;
    use crate::ByFunctionHolder;
    use crate::NodeType;
    use crate::Number;
    use crate::Runtime;
    use crate::AST;
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
            let closure = |x: &Value| match x {
                Value::Number(Number { .. }) => Ok(x.clone()),
                _ => Err(RuntimeError::new(Kind::InvalidType, "err")),
            };
            Ok(ByFunctionHolder {
                closure: Box::new(closure),
            })
        }
    }

    fn setup() -> Fixture {
        let runtime = Runtime::create_runtime();
        Fixture { runtime }
    }

    #[rstest]
    #[case(Err(Kind::InvalidType), Value::Array(vec!["not a number".into()]))]
    #[case(Ok(Value::Null), Value::Array(vec![]))]
    #[case(Ok(Value::from_f64(1.0).unwrap()), Value::Array(vec![3.into(), 1.into(), 2.into()]))]
    fn length(#[case] expected: Result<Value, Kind>, #[case] input: Value) {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        let identifier = AST::make(NodeType::CurrentNode, Position::new(1, 2));
        let expression = AST::make(NodeType::Expression(vec![identifier]), Position::new(1, 1));
        let expref = Value::Expression(expression);

        // call function

        let fname = "min_by";
        let args = vec![input, expref];
        let result = fixture
            .runtime
            .call(fname, &args, context)
            .map_err(|e| e.kind);

        assert_eq!(expected, result);
    }
}
