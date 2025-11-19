use crate::function;

use crate::FunctionContext;
use crate::Value;
use crate::functions::ReturnValue;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(pad_left, [
    subject => Required(Of(DataType::String)),
    width => Required(Of(DataType::Number)),
    pad => Optional(Of(DataType::String))
    ], |me: &pad_left, args: &Vec<Value>, _: &dyn FunctionContext| {

        // parse subject
        let subject_str = args[0].as_str().unwrap();
        let subject: Vec<_> = subject_str.chars().collect();

        // validate width and get usize width
        let width = crate::functions::builtin::PadImpl::width_from_arg(me, &args[1])?;

        // validate pad (even if it may not be used) and get pad char
        let pad_char = crate::functions::builtin::PadImpl::pad_char_from_args(me, args)?;

        if subject.len() >= width {
            return Ok(Value::String(subject_str.to_string()));
        }

        let pad_count = width - subject.len();
        let mut s = String::with_capacity(subject_str.len() + pad_count);
        for _ in 0..pad_count {
            s.push(pad_char);
        }
        s.push_str(subject_str);
        Ok(Value::String(s))
});

// pad_left specific impl is intentionally empty; helpers live in `PadImpl`.

#[cfg(test)]
mod tests {
    use crate::errors::Kind;
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Ok(Value::String("string".to_string())), Value::String("string".to_string()), Value::Number(0.into()), Value::Null)]
    #[case(Ok(Value::String("string".to_string())), Value::String("string".to_string()), Value::Number(5.into()), Value::Null)]
    #[case(Ok(Value::String(format!("{}{}", " ".repeat(4), "string"))), Value::String("string".to_string()), Value::Number(10.into()), Value::Null)]
    #[case(Ok(Value::String("----string".to_string())), Value::String("string".to_string()), Value::Number(10.into()), Value::String("-".to_string()))]
    #[case(Err(Kind::InvalidValue), Value::String("subject".to_string()), Value::Number(1.into()), Value::String("--".to_string()))]
    fn pad_left(
        #[case] expected: Result<Value, Kind>,
        #[case] subject: Value,
        #[case] width: Value,
        #[case] pad: Value,
    ) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        let fname = "pad_left";
        let args = vec![subject, width, pad]
            .into_iter()
            .filter(|x| !x.is_null())
            .collect();
        let result = fixture
            .runtime
            .call(fname, &args, context)
            .map_err(|e| e.kind);

        assert_eq!(expected, result);
    }
}
