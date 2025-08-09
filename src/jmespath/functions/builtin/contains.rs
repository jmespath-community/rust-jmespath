use crate::function;

use crate::FunctionContext;
use crate::Value;
use crate::functions::ReturnValue;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(contains, [
    subject => Required(Any(vec![DataType::Array, DataType::String])),
    search => Required(Of(DataType::Any))
    ], |_: &contains, args: &Vec<Value>, _: &dyn FunctionContext| {
    if let Some(vec) = args[0].as_array() {
        return Ok(Value::Boolean(vec.iter().any(|x| x == &args[1])));
    }
    if let Some(subject_string) = args[0].as_str() {
        if let Some(search_string) = args[1].as_str() {
            let subject: Vec<_> = subject_string.chars().collect();
            let search: Vec<_> = search_string.chars().collect();
            let contains = subject.windows(search.len()).any(|window| window == &search);
            return Ok(contains.into())
        }
        return Ok(false.into())
    }
    unreachable!();
});

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::Boolean(true), Value::from_json(r#"[1, 2]"#).unwrap(), Value::from_f64(1.0).unwrap())]
    #[case(Value::Boolean(false), Value::from_json(r#"[1, 2]"#).unwrap(), Value::from_f64(3.0).unwrap())]
    #[case(Value::Boolean(true), Value::String("substring".to_string()), Value::String("string".to_string()))]
    #[case(Value::Boolean(false), Value::String("substring".to_string()), Value::String("unknown".to_string()))]
    #[case(Value::Boolean(false), Value::String("substring".to_string()), Value::Boolean(true))]
    fn contains(#[case] expected: Value, #[case] subject: Value, #[case] search: Value) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "contains";
        let args = vec![subject, search];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
