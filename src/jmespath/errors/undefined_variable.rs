use super::{Error, Kind, Position, error_builder};

pub(crate) trait UndefinedVariableErrorBuilderFactory {
    type Builder: super::error_builder::UndefinedVariableErrorBuilder;
    fn get_undefined_variable_error_builder() -> Self::Builder;
}
impl self::UndefinedVariableErrorBuilderFactory for Error {
    type Builder = self::UndefinedVariableErrorBuilder;

    fn get_undefined_variable_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}
pub(crate) struct UndefinedVariableErrorBuilder {
    message: String,
    position: Option<Position>,
    variable_name: String,
}
impl UndefinedVariableErrorBuilder {
    pub fn new() -> Self {
        UndefinedVariableErrorBuilder {
            message: "".to_string(),
            position: None,

            variable_name: "".to_string(),
        }
    }
    fn format(&mut self) {
        self.message = format!("the variable '{}' is not defined", self.variable_name)
    }
}
impl error_builder::UndefinedVariableErrorBuilder for UndefinedVariableErrorBuilder {
    fn for_variable(&mut self, name: &str) -> &mut Self {
        self.variable_name = name.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for UndefinedVariableErrorBuilder {
    fn at(&mut self, position: super::Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: Kind::UndefinedVariable,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{ErrorBuilder, UndefinedVariableErrorBuilder};

    use super::*;

    #[test]
    fn undefined_variable_error_builder() {
        let err = Error::get_undefined_variable_error_builder()
            .at(Position::new(1, 7))
            .for_variable("$var")
            .build();

        assert_eq!(
            "Error(1, 7): undefined-variable, the variable '$var' is not defined",
            format!("{}", err)
        );
    }

    #[test]
    fn undefined_variable() {
        let mut err = Error::undefined_variable("param");
        err.position = Some(Position::new(1, 7));
        assert_eq!(
            "Error(1, 7): undefined-variable, the variable 'param' is not defined",
            format!("{}", err)
        );
    }
}
