use super::{error_builder, Error, Kind, Position};

pub(crate) trait SyntaxErrorBuilderFactory {
    type Builder: super::error_builder::SyntaxErrorBuilder;
    fn get_syntax_error_builder() -> Self::Builder;
}
impl self::SyntaxErrorBuilderFactory for Error {
    type Builder = crate::errors::syntax::SyntaxErrorBuilder;

    fn get_syntax_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}
pub struct SyntaxErrorBuilder {
    kind: Kind,
    message: String,
    position: Option<Position>,
    reason: String,
}
impl SyntaxErrorBuilder {
    pub fn new() -> Self {
        SyntaxErrorBuilder {
            kind: Kind::Syntax,
            message: "".to_string(),
            reason: "".to_string(),
            position: None,
        }
    }
    fn format(&mut self) {
        self.message = self.reason.to_string();
    }
}
impl error_builder::SyntaxErrorBuilder for SyntaxErrorBuilder {
    fn set_kind(&mut self, kind: Kind) -> &mut Self {
        self.kind = kind;
        self
    }
    fn for_reason(&mut self, reason: &str) -> &mut Self {
        self.reason = reason.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for SyntaxErrorBuilder {
    fn at(&mut self, position: Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        self.format();
        return Error {
            kind: self.kind,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}
