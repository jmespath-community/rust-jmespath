use super::{error_builder, Error, Kind, Position};

pub(crate) trait UnknownFunctionErrorBuilderFactory {
    type Builder: super::error_builder::UnknownFunctionErrorBuilder;
    fn get_unknown_function_error_builder() -> Self::Builder;
}
impl self::UnknownFunctionErrorBuilderFactory for Error {
    type Builder = self::UnknownFunctionErrorBuilder;

    fn get_unknown_function_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}
pub(crate) struct UnknownFunctionErrorBuilder {
    function_name: String,
    message: String,
    position: Option<Position>,
}
impl UnknownFunctionErrorBuilder {
    pub fn new() -> Self {
        UnknownFunctionErrorBuilder {
            message: "".to_string(),
            position: None,

            function_name: "".to_string(),
        }
    }
    fn format(&mut self) {
        self.message = format!("the function '{}' does not exist", self.function_name)
    }
}
impl error_builder::FunctionErrorBuilder for UnknownFunctionErrorBuilder {
    fn for_function(&mut self, name: &str) -> &mut Self {
        self.function_name = name.to_string();
        self
    }
}
impl error_builder::UnknownFunctionErrorBuilder for UnknownFunctionErrorBuilder {}
impl error_builder::ErrorBuilder for UnknownFunctionErrorBuilder {
    fn at(&mut self, position: Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: Kind::UnknownFunction,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{ErrorBuilder, FunctionErrorBuilder};

    use super::*;

    #[test]
    fn unknown_function_error_builder() {
        let err = Error::get_unknown_function_error_builder()
            .at(Position::new(1, 7))
            .for_function("unknown")
            .build();

        assert_eq!(
            "Error(1, 7): unknown-function, the function 'unknown' does not exist",
            format!("{}", err)
        );
    }
    #[test]
    fn unknown_function() {
        let mut err = Error::unknown_function("unknown");
        err.position = Some(Position::new(1, 7));
        assert_eq!(
            "Error(1, 7): unknown-function, the function 'unknown' does not exist",
            format!("{}", err)
        );
    }
}
