/// Represents a category for an error.
#[derive(Debug, Copy, Clone)]
pub enum Kind {
    /// Represents an error that occurs when too many or too few parameters where passed to a JMESPath function.
    InvalidArity,
    /// Represents an error that occurs when a JMESPath function receives an argument whose type is not expected.
    InvalidType,
    /// Represents an error that occurs when a JMESPath function evaluates an argument whose value does not fall within an expected range.
    InvalidValue,
    /// Represents an error that occurs when performing arithmetic operations on arguments that cannot be evaluated to valid numbers.
    NotANumber,
    /// Represents an error that occurs when trying to evaluate a variable that has not been defined in any bindings.
    UndefinedVariable,
    /// Represents an error that occurs when evaluating a JMESPath function that has not been registered.
    UnknownFunction,
    /// Represents an error that occurs when parsing the JMESPath expression.
    Syntax,
}
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Kind::InvalidArity => "invalid-arity",
            Kind::InvalidType => "invalid-type",
            Kind::InvalidValue => "invalid-value",
            Kind::NotANumber => "not-a-number",
            Kind::UndefinedVariable => "undefined-variable",
            Kind::UnknownFunction => "unknown-function",
            Kind::Syntax => "syntax",
        };
        write!(f, "{}", s)
    }
}

impl Eq for Kind {}
impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
impl PartialOrd for Kind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Kind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        format!("{}", self).cmp(&format!("{}", other))
    }
}

#[cfg(test)]
mod kind_test {
    use crate::errors::kind::Kind;
    use crate::errors::kind::Kind::*;
    use rstest::rstest;
    use std::cmp::Ordering;

    #[rstest]
    #[case("invalid-arity", InvalidArity)]
    #[case("invalid-type", InvalidType)]
    #[case("invalid-value", InvalidValue)]
    #[case("not-a-number", NotANumber)]
    #[case("undefined-variable", UndefinedVariable)]
    #[case("unknown-function", UnknownFunction)]
    #[case("syntax", Syntax)]
    fn it_implements_display_trait(#[case] expected: &str, #[case] kind: Kind) {
        assert_eq!(expected, format!("{}", kind));
    }

    #[rstest]
    #[case(true, Kind::InvalidArity, Kind::InvalidArity)]
    #[case(false, Kind::InvalidArity, Kind::InvalidValue)]
    fn it_implements_eq_trait(#[case] expected: bool, #[case] left: Kind, #[case] right: Kind) {
        assert_eq!(expected, left == right);
    }

    #[rstest]
    #[case(Ordering::Less, Kind::Syntax, Kind::UndefinedVariable)]
    #[case(Ordering::Equal, Kind::Syntax, Kind::Syntax)]
    #[case(Ordering::Greater, Kind::Syntax, Kind::InvalidValue)]
    fn it_implements_partial_ord_trait(
        #[case] expected: Ordering,
        #[case] left: Kind,
        #[case] right: Kind,
    ) {
        assert_eq!(expected, left.partial_cmp(&right).unwrap());
    }
}
