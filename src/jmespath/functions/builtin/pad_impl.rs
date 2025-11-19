use crate::Value;
use crate::errors::Error as RuntimeError;
use crate::errors::error_builder::ErrorBuilder;
use crate::errors::error_builder::FunctionErrorBuilder;
use crate::errors::error_builder::InvalidValueErrorBuilder;
use crate::errors::invalid_value::InvalidValueErrorBuilderFactory;
use crate::functions::Function;

pub(crate) struct PadImpl;
impl PadImpl {
    pub(crate) fn is_integer(number: f64) -> bool {
        number.floor() == number
    }

    pub(crate) fn width_from_arg(me: &dyn Function, arg: &Value) -> Result<usize, RuntimeError> {
        let width = arg.as_number().unwrap();
        if !Self::is_integer(width.number) {
            return Err(RuntimeError::get_invalid_value_error_builder()
                .for_function(me.get_name())
                .for_parameter("width")
                .expected("an integer")
                .received(arg)
                .build());
        }
        if width.number < 0.0 {
            return Err(RuntimeError::get_invalid_value_error_builder()
                .for_function(me.get_name())
                .for_parameter("width")
                .expected("a non-negative integer")
                .received(arg)
                .build());
        }
        Ok(width.number.floor() as usize)
    }

    pub(crate) fn pad_char_from_args(
        me: &dyn Function,
        args: &Vec<Value>,
    ) -> Result<char, RuntimeError> {
        if args.len() > 2 {
            let pad_str = args[2].as_str().unwrap();
            let pad_chars: Vec<_> = pad_str.chars().collect();
            if pad_chars.len() != 1 {
                return Err(RuntimeError::get_invalid_value_error_builder()
                    .for_function(me.get_name())
                    .for_parameter("pad")
                    .expected("a string with length 1")
                    .received(&args[2])
                    .build());
            }
            Ok(pad_chars[0])
        } else {
            Ok(' ')
        }
    }
}
