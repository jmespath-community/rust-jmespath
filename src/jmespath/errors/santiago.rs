use super::{Kind, Position};
use crate::{Error, parser::AST};

impl std::error::Error for Error {}

impl From<santiago::parser::ParseError<AST>> for Error {
    fn from(error: santiago::parser::ParseError<AST>) -> Self {
        let kind = Kind::Syntax;
        // TODO: better error handling
        let message = format!("{:?}", error);
        let position = match &error.at {
            Some(lexeme) => Some(Position::new(lexeme.position.line, lexeme.position.column)),
            None => Some(Position::new(1, 1)),
        };
        Self {
            kind,
            message,
            position,
        }
    }
}
