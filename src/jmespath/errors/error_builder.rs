use crate::{Value, functions::DataType};

use super::{Error, Kind, Position};

pub(crate) trait ErrorBuilder {
    fn at(&mut self, position: Position) -> &mut Self;
    fn build(&mut self) -> Error;
}
pub(crate) trait FunctionErrorBuilder: ErrorBuilder {
    fn for_function(&mut self, name: &str) -> &mut Self;
}
pub(crate) trait InvalidArityErrorBuilder: FunctionErrorBuilder {
    fn min_expected(&mut self, min_count: usize) -> &mut Self;
    fn max_expected(&mut self, max_count: usize) -> &mut Self;
    fn supplied(&mut self, count: usize) -> &mut Self;
    fn variadic(&mut self, is_variadic: bool) -> &mut Self;
}

pub(crate) trait InvalidTypeErrorBuilder: FunctionErrorBuilder {
    fn for_parameter(&mut self, name: &str) -> &mut Self;
    fn for_expression_parameter(&mut self, name: &str) -> &mut Self;
    fn expected_data_types(&mut self, data_types: &Vec<DataType>) -> &mut Self;
    fn received(&mut self, arg: &Value) -> &mut Self;
}
pub(crate) trait InvalidValueErrorBuilder: FunctionErrorBuilder {
    fn for_parameter(&mut self, name: &str) -> &mut Self;
    fn expected(&mut self, expected: &str) -> &mut Self;
    fn received(&mut self, arg: &Value) -> &mut Self;
}
pub(crate) trait NotANumberErrorBuilder: ErrorBuilder {
    fn for_reason(&mut self, message: &str) -> &mut Self;
}
pub(crate) trait SyntaxErrorBuilder: ErrorBuilder {
    fn set_kind(&mut self, kind: Kind) -> &mut Self;
    fn for_reason(&mut self, reason: &str) -> &mut Self;
}
pub(crate) trait UndefinedVariableErrorBuilder: ErrorBuilder {
    fn for_variable(&mut self, name: &str) -> &mut Self;
}
pub(crate) trait UnknownFunctionErrorBuilder: ErrorBuilder {}
