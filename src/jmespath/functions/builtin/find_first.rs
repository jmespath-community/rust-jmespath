use crate::errors::error_builder::FunctionErrorBuilder;
use crate::errors::error_builder::InvalidValueErrorBuilder;
use crate::function;

use crate::errors::Error as RuntimeError;
use crate::errors::error_builder::ErrorBuilder;
use crate::errors::invalid_value::InvalidValueErrorBuilderFactory;

use crate::FunctionContext;
use crate::Number;
use crate::Value;
use crate::functions::ReturnValue;

use crate::functions::DataType;
use crate::functions::Function;
use crate::functions::ParamTypes::*;
use crate::functions::Parameter::{self, *};

function!(find_first, [
    subject => Required(Of(DataType::String)),
    sub => Required(Of(DataType::String)),
    start => Optional(Of(DataType::Number)),
    end => Optional(Of(DataType::Number))
    ], |me: &find_first, args: &Vec<Value>, _: &dyn FunctionContext| {

        let subject: Vec<_> = args[0].as_str().unwrap().chars().collect();
        let sub: Vec<_> = args[1].as_str().unwrap().chars().collect();
        if subject.len() == 0 || sub.len() == 0 {
            return Ok(Value::Null);
        }

        let zero = &Number::from(0.0).unwrap();
        let infinite = &Number::from(subject.len() as f64).unwrap();

        let start = if args.len() > 2 { args[2].as_number().unwrap() } else { zero };
        let end = if args.len() > 3 { args[3].as_number().unwrap() } else { infinite };

        if !Self::is_integer(start.number) {
            return Err(RuntimeError::get_invalid_value_error_builder()
            .for_function(me.get_name())
            .for_parameter("start")
            .expected("an integer")
            .received(&args[2])
            .build());
        }
        if !Self::is_integer(end.number) {
            return Err(RuntimeError::get_invalid_value_error_builder()
            .for_function(me.get_name())
            .for_parameter("end")
            .expected("an integer")
            .received(&args[3])
            .build());
        }

        let start = std::cmp::max(start, zero).number.floor() as usize;
        let end = std::cmp::min(end, infinite).number.floor() as usize;

        let subject = &subject[start..end];
        if let Some(offset) = subject.windows(sub.len()).position(|window| window == sub) {
            return Ok((offset + start).into());
        }
        Ok(Value::Null)
});

impl find_first {
    pub(crate) fn is_integer(number: f64) -> bool {
        number.floor() == number
    }
}

#[cfg(test)]
mod tests {
    use crate::functions::builtin::test_utils::Fixture;
    use crate::{FunctionContext, Value};
    use rstest::*;

    #[rstest]
    #[case(Value::Null, Value::String("subject string".to_string()), Value::String("".to_string()), Value::Null, Value::Null)]
    #[case(Value::Null, Value::String("".to_string()), Value::String("string".to_string()), Value::Null, Value::Null)]
    #[case(Value::Null, Value::String("".to_string()), Value::String("".to_string()), Value::Null, Value::Null)]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Null, Value::Null)]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(0.into()), Value::Null)]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(0.into()), Value::Number(14.into()))]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::from_f64(-6.0).unwrap(), Value::Null)]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::from_f64(-99.0).unwrap(), Value::Number(100.into()))]
    #[case(Value::Null, Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(0.into()), Value::Number(13.into()))]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(8.into()), Value::Null)]
    #[case(Value::Null, Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(8.into()), Value::Number(11.into()))]
    #[case(Value::Null, Value::String("subject string".to_string()), Value::String("string".to_string()), Value::Number(9.into()), Value::Null)]
    #[case(Value::Number(0.into()), Value::String("subject string".to_string()), Value::String("s".to_string()), Value::Null, Value::Null)]
    #[case(Value::Number(8.into()), Value::String("subject string".to_string()), Value::String("s".to_string()), Value::Number(1.into()), Value::Null)]
    fn find_first(
        #[case] expected: Value,
        #[case] subject: Value,
        #[case] sub: Value,
        #[case] start: Value,
        #[case] end: Value,
    ) {
        let fixture = Fixture::setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let fname = "find_first";
        let args = vec![subject, sub, start, end]
            .into_iter()
            .filter(|x| !x.is_null())
            .collect();
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!(expected, result);
    }
}
