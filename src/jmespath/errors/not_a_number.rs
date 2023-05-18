use super::{error_builder, Error, Kind, Position};

pub(crate) trait NotANumberErrorBuilderFactory {
    type Builder: super::error_builder::NotANumberErrorBuilder;
    fn get_not_a_number_error_builder() -> Self::Builder;
}
impl self::NotANumberErrorBuilderFactory for Error {
    type Builder = self::NotANumberErrorBuilder;

    fn get_not_a_number_error_builder() -> Self::Builder {
        Self::Builder::new()
    }
}
pub(crate) struct NotANumberErrorBuilder {
    message: String,
    position: Option<Position>,
}
impl NotANumberErrorBuilder {
    pub fn new() -> Self {
        NotANumberErrorBuilder {
            message: "".to_string(),
            position: None,
        }
    }
}
impl error_builder::NotANumberErrorBuilder for NotANumberErrorBuilder {
    fn for_reason(&mut self, message: &str) -> &mut Self {
        self.message = message.to_string();
        self
    }
}
impl error_builder::ErrorBuilder for NotANumberErrorBuilder {
    fn at(&mut self, position: super::Position) -> &mut Self {
        self.position = Some(position);
        self
    }
    fn build(&mut self) -> Error {
        return Error {
            kind: Kind::NotANumber,
            message: std::mem::replace(&mut self.message, String::new()),
            position: self.position,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error_builder::{ErrorBuilder, NotANumberErrorBuilder};

    use super::*;

    #[test]
    fn not_a_number_error_builder() {
        let err = Error::get_not_a_number_error_builder()
            .at(Position::new(1, 7))
            .for_reason("the expression did not evaluate to a valid number")
            .build();

        assert_eq!(
            "Error(1, 7): not-a-number, the expression did not evaluate to a valid number",
            format!("{}", err)
        );
    }
}
