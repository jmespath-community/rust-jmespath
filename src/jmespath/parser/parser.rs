use super::{grammar, AST};
use crate::errors::Error as ParseError;
use crate::lexer::tokenize;
use santiago::lexer::Lexeme;
use std::rc::Rc;

/// Parses a JMESPath expression and returns an [`AST`]
/// that represents the compiled abstract syntax tree.
///
/// # Example
///
/// ```
/// let expression = "'foo'";
/// let ast = jmespath_community::parse(expression).unwrap();
///
/// assert_eq!("RawString(foo) [1, 1]", format!("{}", ast));
/// ```

pub fn parse(input: &str) -> Result<AST, ParseError> {
    let tokens = tokenize(input)?;
    parse_tokens(tokens)
}
fn parse_tokens(tokens: Vec<Rc<Lexeme>>) -> Result<AST, ParseError> {
    let grammar = grammar::grammar();
    let parse_trees = santiago::parser::parse(&grammar, &tokens)?;
    let parse_tree = parse_trees.get(0);
    return match parse_tree {
        Some(parsed) => Ok(parsed.as_abstract_syntax_tree()),
        None => panic!("something wrong happened"),
    };
}

#[cfg(test)]
mod tests {

    use crate::parser::{parse, NodeType, AST};
    use rstest::*;

    #[test]
    fn error() {
        let ast = parse("foo.@");
        assert!(ast.is_err());
    }

    #[test]
    fn current_node() {
        let ast = parse("@");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::CurrentNode,
                ..
            })
        ));
    }
    #[rstest]
    #[case("\"foo\"", r#"`"foo"`"#)]
    fn json_value(#[case] expected: &str, #[case] input: &str) {
        let ast = parse(input);
        match ast {
            Ok(AST {
                node_type: NodeType::JsonValue(text),
                ..
            }) => assert_eq!(expected, text),
            _ => assert!(false),
        }
    }
    #[rstest]
    #[case(r#""foo""#)]
    #[case(r#""foo\"bar\"""#)]
    #[case(r#""\b\f\n\r\t\/""#)]
    fn quoted_string(#[case] input: &str) {
        let ast = parse(input);
        match ast {
            Ok(AST {
                node_type: NodeType::QuotedIdentifier(text),
                ..
            }) => assert_eq!(input, text),
            _ => assert!(false),
        }
    }
    #[rstest]
    #[case("", "''")]
    #[case("raw_string", "'raw_string'")]
    #[case("\\", r#"'\\'"#)]
    #[case("'", r#"'\''"#)]
    fn raw_string(#[case] expected: &str, #[case] input: &str) {
        let ast = parse(input);
        match ast {
            Ok(AST {
                node_type: NodeType::RawString(text),
                ..
            }) => assert_eq!(expected, text),
            _ => assert!(false),
        }
    }
    #[test]
    fn root_node() {
        let ast = parse("$");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::RootNode,
                ..
            })
        ));
    }
    #[rstest]
    #[case("foo", "foo")]
    fn unquoted_string(#[case] input: &str, #[case] expected: &str) {
        let ast = parse(input);
        match ast {
            Ok(AST {
                node_type: NodeType::UnquotedIdentifier(text),
                ..
            }) => assert_eq!(expected, text),
            _ => assert!(false),
        }
    }

    #[rstest]
    #[case("+ bar")]
    #[case("- bar")]
    #[case("− bar")]
    #[case("- −bar")]
    #[case("+ −bar")]
    fn arithmetic_expression_unary(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::ArithmeticExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("foo + bar")]
    #[case("foo - bar")]
    #[case("foo − bar")]
    #[case("foo × bar")]
    #[case("foo * bar")]
    #[case("foo ÷ bar")]
    #[case("foo / bar")]
    #[case("foo % bar")]
    #[case("foo // bar")]
    fn arithmetic_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::ArithmeticExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("foo < bar")]
    #[case("foo <= bar")]
    #[case("foo == bar")]
    #[case("foo != bar")]
    #[case("foo > bar")]
    #[case("foo >= bar")]
    fn comparison_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::ComparatorExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("custom_function()")]
    #[case("custom_function(one)")]
    #[case("custom_function(one, two)")]
    #[case("custom_function(one, two, three)")]
    #[case("length(foo)")]
    #[case("min_by(foo, &age)")]
    fn function_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::FunctionExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("*")]
    #[case("foo.*")]
    #[case("foo.*.bar")]
    fn hash_wildcard_projection(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::HashWildcardProjection(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("[0]")]
    #[case("foo[0]")]
    fn index_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::IndexExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("let $foo = foo in bar")]
    #[case("let $foo = foo, $bar = bar in baz")]
    fn let_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::LetExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("foo && bar")]
    #[case("foo || bar")]
    #[case("! foo")]
    fn logical_expression(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::LogicalExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    //#[case("{ }")] // non-standard empty multi-select hash
    #[case("{foo: foo }")]
    #[case("{foo: foo, bar: bar }")]
    #[case("{foo: foo, bar: bar, baz: baz }")]
    fn multi_select_hash(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::MultiSelectHash(..),
                ..
            })
        ));
    }
    #[rstest]
    //#[case("[ ]")] // non-standard empty multi-select-list
    #[case("[one]")]
    #[case("[one, two]")]
    #[case("[one, two, three]")]
    fn multi_select_list(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::MultiSelectList(..),
                ..
            })
        ));
    }
    #[test]
    fn paren_expression() {
        let ast = parse("(foo)");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::ParenExpression(..),
                ..
            })
        ))
    }
    #[test]
    fn pipe_expression() {
        let ast = parse("foo | bar");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::PipeExpression(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("[]")]
    #[case("foo[]")]
    #[case("foo[].bar")]
    #[case("[*]")]
    #[case("foo[*]")]
    #[case("[?foo==bar]")]
    #[case("bar[?foo==bar]")]
    #[case("bar[?foo==bar].baz")]
    #[case("[::]")]
    #[case("foo[::]")]
    #[case("foo[::].bar")]
    fn projection(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::Projection(..),
                ..
            })
        ));
    }
    #[rstest]
    #[case("[:]")]
    #[case("[0:]")]
    #[case("[:1]")]
    #[case("[0:1]")]
    #[case("[::]")]
    #[case("[0::]")]
    #[case("[:1:]")]
    #[case("[0:1:]")]
    #[case("[::1]")]
    #[case("[0::1]")]
    #[case("[:1:1]")]
    #[case("[0:1:1]")]
    fn slice(#[case] input: &str) {
        let ast = parse(input);
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::Projection(..),
                ..
            })
        ));
    }
    #[test]
    fn sub_expression() {
        let ast = parse("foo.bar");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::SubExpression(..),
                ..
            })
        ));
    }
    #[test]
    fn variable_ref() {
        let ast = parse("$foo");
        assert!(matches!(
            ast,
            Ok(AST {
                node_type: NodeType::VariableRef(..),
                ..
            })
        ));
    }
}
