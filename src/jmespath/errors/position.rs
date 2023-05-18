/// Represents a location within a JMESPath expression.
#[derive(Debug, Copy, Clone)]
pub struct Position {
    /// The line number, starting at line 1.
    pub line: usize,
    /// The column number, starting at column 1.
    pub column: usize,
}
impl Position {
    /// A default invalid 'null-object' [`Position`].
    pub(crate) fn default() -> Self {
        Self::new(0, 0)
    }
    /// Creates a new instance of the [`Position`] type.
    pub(crate) fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.line, self.column)
    }
}

impl Eq for Position {}
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.column == other.column
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.line < other.line {
            return std::cmp::Ordering::Less;
        }
        if self.line == other.line {
            if self.column < other.column {
                return std::cmp::Ordering::Less;
            }
            if self.column == other.column {
                return std::cmp::Ordering::Equal;
            }
            return std::cmp::Ordering::Greater;
        }
        return std::cmp::Ordering::Greater;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::cmp::Ordering;

    #[rstest]
    #[case("(1, 1)", Position { line: 1, column: 1})]
    fn it_implements_display_trait(#[case] expected: &str, #[case] position: Position) {
        assert_eq!(expected, format!("{}", position));
    }

    #[rstest]
    #[case(true, Position { line: 1, column: 1 }, Position { line: 1, column: 1})]
    #[case(false, Position { line: 1, column: 3 }, Position { line: 1, column: 1})]
    fn it_implements_eq_trait(
        #[case] expected: bool,
        #[case] left: Position,
        #[case] right: Position,
    ) {
        assert_eq!(expected, left == right);
    }

    #[rstest]
    #[case(Ordering::Less, Position { line: 1, column: 3 }, Position { line: 2, column: 4})]
    #[case(Ordering::Less, Position { line: 2, column: 3 }, Position { line: 2, column: 4})]
    #[case(Ordering::Equal, Position { line: 1, column: 1 }, Position { line: 1, column: 1})]
    #[case(Ordering::Greater, Position { line: 2, column: 4 }, Position { line: 1, column: 3})]
    #[case(Ordering::Greater, Position { line: 1, column: 4 }, Position { line: 1, column: 3})]
    fn it_implements_partial_ord_trait(
        #[case] expected: Ordering,
        #[case] left: Position,
        #[case] right: Position,
    ) {
        assert_eq!(expected, left.partial_cmp(&right).unwrap());
    }
}
