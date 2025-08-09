use std::cmp::Ordering;

use crate::errors::invalid_arity::InvalidArityErrorBuilderFactory;
use crate::errors::undefined_variable::UndefinedVariableErrorBuilderFactory;
use crate::errors::unknown_function::UnknownFunctionErrorBuilderFactory;
use crate::errors::Kind;
use crate::errors::Position;

use super::error_builder::ErrorBuilder;
use super::error_builder::FunctionErrorBuilder;
use super::error_builder::InvalidArityErrorBuilder;
use super::error_builder::UndefinedVariableErrorBuilder;

/// The error type for this crate.
#[derive(Debug, Clone)]
pub struct Error {
    /// One of the valid error [`Kind`] values.
    pub kind: Kind,
    /// A message indicating the root cause for the error.
    pub message: String,
    /// The position within a JMESPath expression.
    pub position: Option<Position>,
}
impl Error {
    /// Creates a new instance of the [`Error`] type.
    pub(crate) fn new(kind: Kind, message: &str) -> Self {
        Error {
            kind,
            message: message.to_string(),
            position: None,
        }
    }
    /// Creates a new instance of the [`Error`] type with specified [`Position`].
    #[cfg(test)]
    pub(crate) fn new_at(kind: Kind, message: &str, position: Position) -> Self {
        Error {
            kind,
            message: message.to_string(),
            position: Some(position),
        }
    }
    /// Raises a runtime error when not enough arguments are supplied to a JMESPath [Function](crate::functions::Function).
    ///
    pub fn too_few_arguments(
        function_name: &str,
        min_count: usize,
        count: usize,
        is_variadic: bool,
    ) -> Error {
        super::Error::get_invalid_arity_error_builder()
            .for_function(function_name)
            .min_expected(min_count)
            .supplied(count)
            .variadic(is_variadic)
            .build()
    }
    /// Raises a runtime error when more arguments are supplied
    /// than expected by a JMESPath [Function](crate::functions::Function).
    ///
    pub fn too_many_arguments(function_name: &str, max_count: usize, count: usize) -> Error {
        super::Error::get_invalid_arity_error_builder()
            .for_function(function_name)
            .max_expected(max_count)
            .supplied(count)
            .build()
    }
    /// Raises a runtime error when an undefined variable is evaluated.
    pub fn undefined_variable(variable_name: &str) -> Error {
        super::Error::get_undefined_variable_error_builder()
            .for_variable(variable_name)
            .build()
    }
    /// Raises a runtime error when an unknown function is invoked.
    ///
    /// # Example
    /// ```
    /// use jmespath_community as jmespath;
    /// use jmespath::errors::Kind;
    /// use jmespath::errors::Error;
    ///
    /// let err = Error::unknown_function("unknown");
    ///
    /// assert_eq!(Kind::UnknownFunction, err.kind);
    /// assert_eq!("Error: unknown-function, the function 'unknown' does not exist", err.to_string());
    /// ```
    pub fn unknown_function(function_name: &str) -> Error {
        super::Error::get_unknown_function_error_builder()
            .for_function(function_name)
            .build()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let position = self.position.map_or("".to_string(), |p| format!("{}", p));
        let kind = format!("Error{}: {}, ", position, self.kind);
        write!(f, "{kind}{}", self.message)
    }
}
impl Eq for Error {}
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        let eq_positions = if self.position.is_some() && other.position.is_some() {
            self.position.unwrap().eq(&other.position.unwrap())
        } else {
            true
        };
        self.kind == other.kind && self.message == other.message && eq_positions
    }
}
impl Ord for Error {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind.cmp(&other.kind) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.message.cmp(&other.message) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        Ordering::Equal
    }
}
impl PartialOrd for Error {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::cmp::Ordering;

    #[rstest]
    #[case("Error: syntax, reason", Error::new(Kind::Syntax, "reason"))]
    #[case("Error(1, 1): syntax, reason", Error::new_at(Kind::Syntax, "reason", Position { line: 1, column: 1, }))]
    fn it_implements_display_trait(#[case] expected: &str, #[case] error: Error) {
        assert_eq!(expected, format!("{}", error));
    }

    #[rstest]
    #[case(
        true,
        Error::new(Kind::Syntax, "reason"),
        Error::new(Kind::Syntax, "reason")
    )]
    #[case(
        true,
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1)),
        Error::new(Kind::Syntax, "reason")
    )]
    #[case(
        true,
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1)),
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1))
    )]
    #[case(
        false,
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1)),
        Error::new_at(Kind::Syntax, "reason", Position::new(2, 1))
    )]
    fn it_implements_eq_trait(#[case] expected: bool, #[case] left: Error, #[case] right: Error) {
        assert_eq!(expected, left == right);
    }

    #[rstest]
    #[case(
        Ordering::Less,
        Error::new_at(Kind::InvalidArity, "reason", Position::new(1, 1)),
        Error::new(Kind::InvalidArity, "some other reason")
    )]
    #[case(
        Ordering::Equal,
        Error::new(Kind::Syntax, "reason"),
        Error::new(Kind::Syntax, "reason")
    )]
    #[case(
        Ordering::Equal,
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1)),
        Error::new(Kind::Syntax, "reason")
    )]
    #[case(
        Ordering::Greater,
        Error::new_at(Kind::Syntax, "reason", Position::new(1, 1)),
        Error::new(Kind::InvalidValue, "reason")
    )]
    fn it_implements_partial_ord_trait(
        #[case] expected: Ordering,
        #[case] left: Error,
        #[case] right: Error,
    ) {
        assert_eq!(expected, left.partial_cmp(&right).unwrap());
    }

    //    use crate::{
    //        errors::{
    //            error_builder::{ErrorBuilder, FunctionErrorBuilder, InvalidTypeErrorBuilder},
    //            invalid_type::InvalidTypeErrorBuilderFactory,
    //            Error,
    //        },
    //        functions::DataType,
    //        Value,
    //    };
    //
    //    #[test]
    //    fn invalid_type() {
    //        let err = Error::get_invalid_type_error_builder()
    //            .for_function("third_party")
    //            .for_expression_parameter("expr")
    //            .expected_data_types(&vec![DataType::Number, DataType::String])
    //            .received(&Value::Null)
    //            .build();
    //
    //        assert_eq!(
    //            "Error: invalid-type, while calling function 'third_party', the expression parameter '$expr' is expected to be expression->[number|string] but the expression evaluated to 'null' (of type null) instead",
    //            err.to_string()
    //        )
    //    }
    //    #[test]
    //    fn invalid_arity_too_many_arguments() {
    //        let err = Error::too_many_arguments("min_by", 2, 3);
    //        assert_eq!(
    //            "Error: invalid-arity, the function 'min_by' expects at most 2 arguments but 3 were specified",
    //            err.to_string()
    //        )
    //    }
    //    #[test]
    //    fn invalid_arity_too_few_arguments() {
    //        let err = Error::too_few_arguments("length", 2, 1, false);
    //        assert_eq!(
    //            "Error: invalid-arity, the function 'length' expects 2 arguments but only 1 were specified",
    //            err.to_string()
    //        )
    //    }
    //    #[test]
    //    fn syntax() {
    //        let err = Error::syntax("it failed");
    //        assert_eq!("Error: syntax, it failed", err.to_string())
    //    }
    //    #[test]
    //    fn undefined_variable() {
    //        let err = Error::undefined_variable("$unknown");
    //        assert_eq!(
    //            "Error: undefined-variable, the variable '$unknown' is not defined",
    //            err.to_string()
    //        )
    //    }
    //    #[test]
    //    fn unknown_function() {
    //        let err = Error::unknown_function("unknown");
    //        assert_eq!(
    //            "Error: unknown-function, the function 'unknown' does not exist",
    //            err.to_string()
    //        )
    //    }
}
