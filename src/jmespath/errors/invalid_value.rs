use crate::{functions::DataType, Value};

use super::{error_builder, Error, Kind, Position};

pub(crate) trait InvalidValueErrorBuilderFactory {
    type Builder: super::error_builder::InvalidValueErrorBuilder;
    fn get_invalid_value_error_builder() -> Self::Builder;
}
impl self::InvalidValueErrorBuilderFactory for Error {
    type Builder = self::InvalidValueErrorBuilder;

    fn get_invalid_value_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}

pub(crate) struct InvalidValueErrorBuilder {
    expected: String,
    function_name: String,
    message: String,
    parameter_name: String,
    position: Option<Position>,
    received_value: Option<Value>,
}
impl InvalidValueErrorBuilder {
    pub fn new() -> Self {
        InvalidValueErrorBuilder {
            expected: "".to_string(),
            function_name: "".to_string(),
            message: "".to_string(),
            parameter_name: "".to_string(),
            position: None,
            received_value: None,
        }
    }
    fn format(&mut self) {
        if let Some(value) = &self.received_value {
            let data_type = value.get_data_type();
            self.message = format!(
                "while calling function '{}', the parameter '${}' evalutated to '{}' (of type {}): expected {} instead",
                self.function_name, self.parameter_name, value, data_type, self.expected
            );
        } else {
            self.message = format!(
                "while calling function '{}', the value for parameter '${}' is invalid: expected {} instead",
                self.function_name, self.parameter_name, self.expected
            );
        }
    }
}
impl error_builder::FunctionErrorBuilder for InvalidValueErrorBuilder {
    fn for_function(&mut self, name: &str) -> &mut Self {
        self.function_name = name.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for InvalidValueErrorBuilder {
    fn at(&mut self, position: super::Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: Kind::InvalidValue,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}
impl error_builder::InvalidValueErrorBuilder for InvalidValueErrorBuilder {
    fn for_parameter(&mut self, name: &str) -> &mut Self {
        self.parameter_name = name.to_string();
        self
    }
    fn received(&mut self, arg: &Value) -> &mut Self {
        self.received_value = Some(arg.clone());
        self
    }
    fn expected(&mut self, reason: &str) -> &mut Self {
        self.expected = reason.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{
        ErrorBuilder, FunctionErrorBuilder, InvalidValueErrorBuilder,
    };

    use super::*;

    #[test]
    fn invalid_value() {
        let err = Error::get_invalid_value_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_parameter("param")
            .expected("an integer less than 2.0")
            .build();

        assert_eq!("Error(1, 4): invalid-value, while calling function 'my_function', the value for parameter '$param' is invalid: expected an integer less than 2.0 instead", format!("{}", err));
    }
    #[test]
    fn invalid_value_received_value() {
        let err = Error::get_invalid_value_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_parameter("param")
            .received(&Value::from_f64(42.0).unwrap())
            .expected("an integer less than 2.0")
            .build();

        assert_eq!("Error(1, 4): invalid-value, while calling function 'my_function', the parameter '$param' evalutated to '42.0' (of type number): expected an integer less than 2.0 instead", format!("{}", err));
    }
}
