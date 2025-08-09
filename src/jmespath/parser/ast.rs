use super::NodeType;
use crate::{Map, errors::Position, functions::ReturnValue};

/// Represents an abstract syntax tree node.
#[derive(Clone)]
pub struct AST {
    /// The node type.
    pub node_type: NodeType,
    /// The position of this node in the JMESPath expression.
    pub position: Position,
}
impl AST {
    /// Creates a new instance of the [`AST`] type.
    pub fn make(node_type: NodeType, position: Position) -> Self {
        AST {
            node_type,
            position,
        }
    }
    /// Evaluates a JMESPath expression.
    ///
    /// # Example
    /// ```
    /// use jmespath_community as jmespath;
    /// use jmespath::{parse, Value};
    /// use jmespath::errors::Position;
    ///
    /// let ast = parse("foo").unwrap();
    /// let data = Value::from_json(r#"{"foo": "bar"}"#).unwrap();
    /// let result = ast.search(&data).unwrap();
    ///
    /// assert_eq!("bar", result);
    /// ```
    pub fn search(&self, root: &crate::Value) -> ReturnValue {
        let runtime = crate::Runtime::get_shared_runtime();
        runtime.search_ast(self, root)
    }
}

impl std::fmt::Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", self.position, self.node_type)
    }
}
impl std::fmt::Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

macro_rules! ast_ {
    ($expr:ident, $enum:ident, $type:ty) => {
        pub(crate) fn $expr(&self) -> $type {
            match self {
                Self {
                    node_type: NodeType::$enum(v),
                    ..
                } => v,
                _ => unreachable!(),
            }
        }
    };
}
macro_rules! pretty {
    ($ident:ident) => {
        (stringify!($ident).to_string(), "".to_string())
    };
    ($ident:ident, $text:expr) => {
        (format!("{}({})", stringify!($ident), $text), "".to_string())
    };
}
impl AST {
    ast_!(hashmap, MultiSelectHash, &Map<String, AST>);
    ast_!(function_name, UnquotedIdentifier, &String);
    ast_!(raw_string, RawString, &String);
    ast_!(variable_ref, VariableRef, &String);
    ast_!(bindings, LetBindings, &Vec<AST>);
    ast_!(function_arguments, FunctionArguments, &Vec<AST>);
    pub(crate) fn identifier(&self) -> &String {
        match &self.node_type {
            NodeType::UnquotedIdentifier(v) => v,
            NodeType::QuotedIdentifier(v) => v,
            _ => unreachable!(),
        }
    }
    pub(crate) fn number(&self) -> i32 {
        match self {
            AST {
                node_type: NodeType::Number(n),
                ..
            } => *n,
            _ => unreachable!(),
        }
    }
    fn format(&self) -> String {
        self.pretty_print(0)
    }
    fn pretty_print(&self, indent_level: usize) -> String {
        fn pretty_print_children(children: &Vec<AST>, indent_level: usize) -> String {
            let mut text = "".to_string();
            for child in children {
                let pretty = format!("\n{}", child.pretty_print(indent_level));
                text.push_str(&pretty);
            }
            text
        }
        fn pretty_print_children_ref(children: &[&AST], indent_level: usize) -> String {
            let mut text = "".to_string();
            for child in children {
                let pretty = format!("\n{}", child.pretty_print(indent_level));
                text.push_str(&pretty);
            }
            text
        }
        let (node_type, children) = match &self.node_type {
            NodeType::And => pretty!(And),
            NodeType::CurrentNode => pretty!(CurrentNode),
            NodeType::Div => pretty!(Div),
            NodeType::Divide => pretty!(Divide),
            NodeType::Equal => pretty!(Equal),
            NodeType::Flatten => pretty!(Flatten),
            NodeType::GreaterThan => pretty!(GreaterThan),
            NodeType::GreaterThanOrEqual => pretty!(GreaterThanOrEqual),
            NodeType::LessThan => pretty!(LessThan),
            NodeType::LessThanOrEqual => pretty!(LessThanOrEqual),
            NodeType::ListWildcard => pretty!(ListWildcard),
            NodeType::Minus => pretty!(Minus),
            NodeType::Modulo => pretty!(Modulo),
            NodeType::Multiply => pretty!(Multiply),
            NodeType::None => pretty!(None),
            NodeType::Not => pretty!(Not),
            NodeType::NotEqual => pretty!(NotEqual),
            NodeType::Or => pretty!(Or),
            NodeType::Plus => pretty!(Plus),
            NodeType::RootNode => pretty!(RootNode),

            NodeType::JsonValue(text) => pretty!(JsonValue, text),
            NodeType::Number(text) => pretty!(Number, text),
            NodeType::QuotedIdentifier(text) => pretty!(QuotedIdentifier, text),
            NodeType::RawString(text) => pretty!(RawString, text),
            NodeType::UnquotedIdentifier(text) => pretty!(UnquotedIdentifier, text),
            NodeType::VariableRef(text) => pretty!(VariableRef, text),

            NodeType::ArithmeticExpression(vec) => (
                "ArithmeticExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::ComparatorExpression(vec) => (
                "ComparatorExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),

            NodeType::Expression(vec) => (
                "Expression".to_string(),
                pretty_print_children_ref(&[&vec[0]], indent_level + 1),
            ),
            NodeType::Filter(vec) => (
                "Filter".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::FunctionArguments(vec) => (
                "FunctionArguments".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::FunctionExpression(vec) => (
                "FunctionExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::HashWildcardProjection(vec) => (
                "HashWildcardProjection".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::Index(vec) => (
                "Index".to_string(),
                pretty_print_children(vec, indent_level),
            ),
            NodeType::IndexExpression(vec) => (
                "IndexExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::LetBindings(vec) => {
                let mut children = "".to_string();
                for index in 0..vec.len() {
                    if index % 2 != 0 {
                        continue;
                    }
                    children.push_str(&pretty_print_children_ref(&[&vec[index]], indent_level + 1));
                    children.push_str(&pretty_print_children_ref(
                        &[&vec[index + 1]],
                        indent_level + 2,
                    ));
                }
                ("LetBindings".to_string(), children)
            }
            NodeType::LetExpression(vec) => (
                "LetExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::LogicalExpression(vec) => (
                "LogicalExpression".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::MultiSelectHash(vec) => {
                let mut children = "".to_string();
                for item in vec {
                    children.push_str(&format!("\n{}{}", "  ".repeat(indent_level + 1), item.0));
                    children.push_str(&pretty_print_children_ref(&[item.1], indent_level + 2));
                }
                ("MultiSelectHash".to_string(), children)
            }
            NodeType::MultiSelectList(vec) => (
                "MultiSelectList".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::ParenExpression(vec) => (
                "ParenExpression".to_string(),
                pretty_print_children_ref(&[&vec[0]], indent_level + 1),
            ),
            NodeType::PipeExpression(vec) => (
                "PipeExpression".to_string(),
                pretty_print_children_ref(&[&vec[0], &vec[1]], indent_level + 1),
            ),
            NodeType::Projection(vec) => (
                "Projection".to_string(),
                pretty_print_children(vec, indent_level + 1),
            ),
            NodeType::Slice(vec) => (
                format!("Slice[{:?}:{:?}:{:?}]", vec.start, vec.stop, vec.step),
                "".to_string(),
            ),
            NodeType::SubExpression(vec) => {
                let children = format!(
                    "\n{}\n{}",
                    vec[0].pretty_print(indent_level + 1),
                    vec[1].pretty_print(indent_level + 1)
                );
                ("SubExpression".to_string(), children)
            }
        };
        format!(
            "{}{} [{}, {}]{}",
            "  ".repeat(indent_level),
            node_type,
            self.position.line,
            self.position.column,
            children,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::NodeType;

    use super::*;
    use rstest::*;

    #[test]
    fn it_implements_debug_trait() {
        let node = NodeType::UnquotedIdentifier("foo".to_string());
        let ast = AST::make(node, Position::new(1, 1));
        assert_eq!("(1, 1):UnquotedIdentifier(\"foo\")", format!("{:?}", ast));
    }
    #[test]
    fn it_implements_display_trait() {
        let node = NodeType::UnquotedIdentifier("foo".to_string());
        let ast = AST::make(node, Position::new(1, 1));
        assert_eq!("UnquotedIdentifier(foo) [1, 1]", format!("{}", ast));
    }

    #[test]
    fn it_pretty_prints_lexeme() {
        let node = NodeType::UnquotedIdentifier("foo".to_string());
        let ast = AST::make(node, Position::new(1, 1));
        assert_eq!("UnquotedIdentifier(foo) [1, 1]", ast.format());
    }
    #[rstest]
    #[case("And [0, 0]", NodeType::And)]
    #[case("CurrentNode [0, 0]", NodeType::CurrentNode)]
    #[case("Div [0, 0]", NodeType::Div)]
    #[case("Divide [0, 0]", NodeType::Divide)]
    #[case("Equal [0, 0]", NodeType::Equal)]
    #[case("Flatten [0, 0]", NodeType::Flatten)]
    #[case("GreaterThan [0, 0]", NodeType::GreaterThan)]
    #[case("GreaterThanOrEqual [0, 0]", NodeType::GreaterThanOrEqual)]
    #[case("LessThan [0, 0]", NodeType::LessThan)]
    #[case("LessThanOrEqual [0, 0]", NodeType::LessThanOrEqual)]
    #[case("ListWildcard [0, 0]", NodeType::ListWildcard)]
    #[case("Minus [0, 0]", NodeType::Minus)]
    #[case("Modulo [0, 0]", NodeType::Modulo)]
    #[case("Multiply [0, 0]", NodeType::Multiply)]
    #[case("None [0, 0]", NodeType::None)]
    #[case("None [0, 0]", NodeType::None)]
    #[case("Not [0, 0]", NodeType::Not)]
    #[case("NotEqual [0, 0]", NodeType::NotEqual)]
    #[case("Or [0, 0]", NodeType::Or)]
    #[case("Plus [0, 0]", NodeType::Plus)]
    #[case("RootNode [0, 0]", NodeType::RootNode)]
    fn it_pretty_prints_lexemes_simple(#[case] expected: &str, #[case] node_type: NodeType) {
        let ast = AST::make(node_type, Position::default());
        assert_eq!(expected, ast.format());
    }
    #[rstest]
    #[case(r#"JsonValue({"foo": "bar"}) [0, 0]"#, NodeType::JsonValue(r#"{"foo": "bar"}"#.to_string()))]
    #[case("Number(42) [0, 0]", NodeType::Number(42))]
    #[case("QuotedIdentifier(foo bar) [0, 0]", NodeType::QuotedIdentifier("foo bar".to_string()))]
    #[case("RawString(raw string) [0, 0]", NodeType::RawString("raw string".to_string()))]
    #[case(
        "UnquotedIdentifier(foo) [0, 0]",
        NodeType::UnquotedIdentifier("foo".to_string())
    )]
    #[case("VariableRef($foo) [0, 0]", NodeType::VariableRef("$foo".to_string()))]
    fn it_pretty_prints_lexemes_text(#[case] expected: &str, #[case] node_type: NodeType) {
        let ast = AST::make(node_type, Position::default());
        assert_eq!(expected, ast.format());
    }

    #[rstest]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Div [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Div
    )]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Divide [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Divide
    )]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Minus [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Minus
    )]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Modulo [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Modulo
    )]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Multiply [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Multiply
    )]
    #[case(
        r#"ArithmeticExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  Plus [1, 4]
  UnquotedIdentifier(bar) [1, 5]"#,
        NodeType::Plus
    )]
    fn it_pretty_prints_arithmetic_expression(#[case] expected: &str, #[case] node_type: NodeType) {
        let nodes = vec![
            AST::make(
                NodeType::UnquotedIdentifier("foo".to_string()),
                Position::new(1, 1),
            ),
            AST::make(node_type, Position::new(1, 4)),
            AST::make(
                NodeType::UnquotedIdentifier("bar".to_string()),
                Position::new(1, 5),
            ),
        ];
        let ast = AST::make(NodeType::ArithmeticExpression(nodes), Position::new(1, 4));
        assert_eq!(expected, ast.format());
    }
    #[test]
    fn it_pretty_prints_pipe_expression() {
        let nodes = vec![
            AST::make(
                NodeType::UnquotedIdentifier("foo".to_string()),
                Position::new(1, 1),
            ),
            AST::make(
                NodeType::UnquotedIdentifier("bar".to_string()),
                Position::new(1, 5),
            ),
        ];
        let ast = AST::make(NodeType::PipeExpression(nodes), Position::new(1, 4));
        assert_eq!(
            r#"PipeExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  UnquotedIdentifier(bar) [1, 5]"#,
            ast.format()
        );
    }

    #[test]
    fn it_pretty_prints_sub_expression() {
        let nodes = vec![
            AST::make(
                NodeType::UnquotedIdentifier("foo".to_string()),
                Position::new(1, 1),
            ),
            AST::make(
                NodeType::UnquotedIdentifier("bar".to_string()),
                Position::new(1, 5),
            ),
        ];
        let ast = AST::make(NodeType::SubExpression(nodes), Position::new(1, 4));
        assert_eq!(
            r#"SubExpression [1, 4]
  UnquotedIdentifier(foo) [1, 1]
  UnquotedIdentifier(bar) [1, 5]"#,
            ast.format()
        );
    }
}
