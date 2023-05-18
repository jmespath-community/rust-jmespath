use crate::{functions::DataType, Value};

use super::{error_builder, Error, Kind, Position};

pub(crate) trait InvalidTypeErrorBuilderFactory {
    type Builder: super::error_builder::InvalidTypeErrorBuilder;
    fn get_invalid_type_error_builder() -> Self::Builder;
}
impl self::InvalidTypeErrorBuilderFactory for Error {
    type Builder = self::InvalidTypeErrorBuilder;

    fn get_invalid_type_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}

pub(crate) struct InvalidTypeErrorBuilder {
    message: String,
    expected_data_types: Vec<DataType>,
    function_name: String,
    parameter_name: String,
    position: Option<Position>,
    received_value: Value,
    received_data_type: DataType,
    is_expref: bool,
}
impl InvalidTypeErrorBuilder {
    pub fn new() -> Self {
        InvalidTypeErrorBuilder {
            message: "".to_string(),
            position: None,

            function_name: "".to_string(),
            parameter_name: "".to_string(),
            expected_data_types: Vec::new(),
            received_value: Value::Null,
            received_data_type: DataType::String,
            is_expref: false,
        }
    }
    fn format(&mut self) {
        if self.is_expref {
            self.format_for_expression_parameter();
        } else {
            self.format_for_parameter();
        }
    }
    fn format_for_expression_parameter(&mut self) {
        assert!(self.expected_data_types.len() > 0);
        let data_types = self
            .expected_data_types
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("|");
        let data_types_list = if self.expected_data_types.len() == 1 {
            format!("expression->{}", data_types)
        } else {
            format!("expression->[{}]", data_types)
        };
        self.message = format!("while calling function '{}', the expression parameter '${}' is expected to be {} but the expression evaluated to '{}' (of type {}) instead",
            self.function_name,
            self.parameter_name,
            data_types_list,
            self.received_value,
            self.received_data_type);
    }
    fn format_for_parameter(&mut self) {
        assert!(self.expected_data_types.len() > 0);
        let data_types = self
            .expected_data_types
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let data_types_list = if self.expected_data_types.len() == 1 {
            format!("{}", data_types)
        } else {
            format!("either one of [{}]", data_types)
        };
        self.message = format!("while calling function '{}', the parameter '${}' is expected to be {} but the value '{}' (of type {}) was received instead",
            self.function_name,
            self.parameter_name,
            data_types_list,
            self.received_value,
            self.received_data_type);
    }
}
impl error_builder::FunctionErrorBuilder for InvalidTypeErrorBuilder {
    fn for_function(&mut self, name: &str) -> &mut Self {
        self.function_name = name.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for InvalidTypeErrorBuilder {
    fn at(&mut self, position: super::Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: Kind::InvalidType,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}
impl error_builder::InvalidTypeErrorBuilder for InvalidTypeErrorBuilder {
    fn for_parameter(&mut self, name: &str) -> &mut Self {
        self.parameter_name = name.to_string();
        self
    }
    fn for_expression_parameter(&mut self, name: &str) -> &mut Self {
        self.parameter_name = name.to_string();
        self.is_expref = true;
        self
    }
    fn expected_data_types(&mut self, data_types: &Vec<DataType>) -> &mut Self {
        for data_type in data_types {
            self.expected_data_types.push(*data_type);
        }
        self
    }
    fn received(&mut self, arg: &Value) -> &mut Self {
        self.received_value = arg.clone();
        self.received_data_type = arg.get_data_type();
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{
        ErrorBuilder, FunctionErrorBuilder, InvalidTypeErrorBuilder,
    };

    use super::*;

    #[test]
    fn invalid_type() {
        let err = Error::get_invalid_type_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_parameter("param")
            .expected_data_types(&vec![DataType::String])
            .received(&Value::from_f64(42.0).unwrap())
            .build();

        assert_eq!("Error(1, 4): invalid-type, while calling function 'my_function', the parameter '$param' is expected to be string but the value '42.0' (of type number) was received instead", format!("{}", err));
    }
    #[test]
    fn invalid_type_any() {
        let err = Error::get_invalid_type_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_parameter("param")
            .expected_data_types(&vec![DataType::Number, DataType::String])
            .received(&Value::from_f64(42.0).unwrap())
            .build();

        assert_eq!("Error(1, 4): invalid-type, while calling function 'my_function', the parameter '$param' is expected to be either one of [number, string] but the value '42.0' (of type number) was received instead", format!("{}", err));
    }
    #[test]
    fn invalid_type_expref() {
        let err = Error::get_invalid_type_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_expression_parameter("expr")
            .expected_data_types(&vec![DataType::String])
            .received(&Value::from_f64(42.0).unwrap())
            .build();

        assert_eq!("Error(1, 4): invalid-type, while calling function 'my_function', the expression parameter '$expr' is expected to be expression->string but the expression evaluated to '42.0' (of type number) instead", format!("{}", err));
    }
    #[test]
    fn invalid_type_expref_any() {
        let err = Error::get_invalid_type_error_builder()
            .at(Position::new(1, 4))
            .for_function("my_function")
            .for_expression_parameter("expr")
            .expected_data_types(&vec![DataType::Number, DataType::String])
            .received(&Value::from_f64(42.0).unwrap())
            .build();

        assert_eq!("Error(1, 4): invalid-type, while calling function 'my_function', the expression parameter '$expr' is expected to be expression->[number|string] but the expression evaluated to '42.0' (of type number) instead", format!("{}", err));
    }
}
