use super::{Error, Kind, Position, error_builder};

pub(crate) trait InvalidArityErrorBuilderFactory {
    type Builder: super::error_builder::InvalidArityErrorBuilder;
    fn get_invalid_arity_error_builder() -> Self::Builder;
}
impl self::InvalidArityErrorBuilderFactory for Error {
    type Builder = self::InvalidArityErrorBuilder;

    fn get_invalid_arity_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}
pub(crate) struct InvalidArityErrorBuilder {
    count: usize,
    function_name: String,
    is_variadic: bool,
    max_count: Option<usize>,
    message: String,
    min_count: Option<usize>,
    position: Option<Position>,
}
impl InvalidArityErrorBuilder {
    pub fn new() -> Self {
        InvalidArityErrorBuilder {
            message: "".to_string(),
            position: None,

            function_name: "".to_string(),
            count: 0,
            is_variadic: false,
            max_count: None,
            min_count: None,
        }
    }
    pub fn format(&mut self) {
        let actual = self.count;
        if let Some(min) = self.min_count {
            let more = if self.is_variadic { "or more " } else { "" };
            let only_actual = format!("only {}", actual);
            let specified = if self.count == 0 {
                "none"
            } else {
                &only_actual
            };
            let plural = if min > 1 { "s" } else { "" };
            self.message = format!(
                "the function '{}' expects {} argument{} {}but {} were specified",
                self.function_name, min, plural, more, specified
            );
        }
        if let Some(max) = self.max_count {
            let plural = if max > 1 { "s" } else { "" };
            self.message = format!(
                "the function '{}' expects at most {} argument{} but {} were specified",
                self.function_name, max, plural, actual
            );
        }
    }
}
impl error_builder::FunctionErrorBuilder for InvalidArityErrorBuilder {
    fn for_function(&mut self, name: &str) -> &mut Self {
        self.function_name = name.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for InvalidArityErrorBuilder {
    fn at(&mut self, position: Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: Kind::InvalidArity,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}
impl error_builder::InvalidArityErrorBuilder for InvalidArityErrorBuilder {
    fn min_expected(&mut self, min_count: usize) -> &mut Self {
        self.min_count = Some(min_count);
        self
    }
    fn max_expected(&mut self, max_count: usize) -> &mut Self {
        self.max_count = Some(max_count);
        self
    }
    fn supplied(&mut self, count: usize) -> &mut Self {
        self.count = count;
        self
    }
    fn variadic(&mut self, is_variadic: bool) -> &mut Self {
        self.is_variadic = is_variadic;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{
        ErrorBuilder, FunctionErrorBuilder, InvalidArityErrorBuilder,
    };

    use super::*;

    #[test]
    fn invalid_arity_error_builder() {
        let err = Error::get_invalid_arity_error_builder()
            .at(Position::new(1, 7))
            .for_function("sum")
            .min_expected(2)
            .variadic(true)
            .build();
        assert_eq!(
            "Error(1, 7): invalid-arity, the function 'sum' expects 2 arguments or more but none were specified",
            format!("{}", err)
        );
    }

    #[test]
    fn too_few_arguments() {
        let mut err = Error::too_few_arguments("add", 2, 1, false);
        err.position = Some(Position::new(1, 7));
        assert_eq!(
            "Error(1, 7): invalid-arity, the function 'add' expects 2 arguments but only 1 were specified",
            format!("{}", err)
        );
    }
    #[test]
    fn too_many_arguments() {
        let mut err = Error::too_many_arguments("mul", 2, 3);
        err.position = Some(Position::new(1, 7));
        assert_eq!(
            "Error(1, 7): invalid-arity, the function 'mul' expects at most 2 arguments but 3 were specified",
            format!("{}", err)
        );
    }
}
