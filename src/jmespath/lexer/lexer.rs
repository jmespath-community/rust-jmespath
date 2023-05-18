use std::rc::Rc;

use santiago::lexer::Lexeme;

use crate::errors::Error as LexerError;
use crate::errors::Position;

use crate::errors::error_builder::ErrorBuilder;
use crate::errors::error_builder::SyntaxErrorBuilder;
use crate::errors::syntax::SyntaxErrorBuilderFactory;
use crate::lexer::rules::lexer_rules;

pub(crate) fn tokenize(input: &str) -> Result<Vec<Rc<Lexeme>>, LexerError> {
    let lexer_rules = lexer_rules();
    let result = santiago::lexer::lex(&lexer_rules, &input);
    return match result {
        Ok(tokens) => Ok(tokens),
        Err(error) => {
            let byte = input.as_bytes()[error.byte_index];
            let extract = String::from_utf8(input.as_bytes()[error.byte_index..].to_vec()).unwrap();
            let category = match byte {
                34 => "quoted-string",
                39 => "raw-string",
                96 => "JSON literal",
                _ => "expression",
            };
            let reason = format!("invalid {} near ->{}<-", category, extract);
            Err(LexerError::get_syntax_error_builder()
                .at(Position::new(error.position.line, error.position.column))
                .for_reason(&reason)
                .build())
        }
    };
}

#[cfg(test)]
mod tests {

    use crate::errors::{Kind, Position};
    use crate::lexer::tokenize;
    use rstest::*;
    use santiago::lexer::Lexeme;
    use std::rc::Rc;

    #[rstest]
    // tokens
    #[case("assign", "=")]
    #[case("colon", ":")]
    #[case("comma", ",")]
    #[case("dot", ".")]
    #[case("pipe", "|")]
    #[case("lparen", "(")]
    #[case("rparen", ")")]
    #[case("lbrace", "{")]
    #[case("rbrace", "}")]
    #[case("lbracket", "[")]
    #[case("rbracket", "]")]
    #[case("filter", "[?")]
    #[case("flatten", "[]")]
    #[case("star", "*")]
    #[case("current", "@")]
    #[case("root", "$")]
    #[case("expref", "&")]
    // arithmetic operators
    #[case("plus", "+")]
    #[case("minus", "-")]
    #[case("minus", "−")]
    #[case("multiply", "×")]
    #[case("divide", "÷")]
    #[case("divide", "/")]
    #[case("div", "//")]
    #[case("mod", "%")]
    // comparison operators
    #[case("equal", "==")]
    #[case("not_equal", "!=")]
    #[case("less_than", "<")]
    #[case("greater_than", ">")]
    #[case("less_than_or_equal", "<=")]
    #[case("greater_than_or_equal", ">=")]
    //logical operators
    #[case("and", "&&")]
    #[case("or", "||")]
    #[case("not", "!")]
    // number
    #[case("number", "42")]
    #[case("number", "-4")]
    // identifiers
    #[case("quoted_string", r#""quoted_string""#)]
    #[case("unquoted_string", "foo")]
    // literals
    #[case("raw_string", "''")]
    #[case("raw_string", "'raw_string'")]
    #[case("raw_string", "' \\\\raw\\\\ '")]
    #[case("json_value", "`true`")]
    #[case("json_value", "`false`")]
    #[case("json_value", "`[1, 2, 3]`")]
    #[case("json_value", r#"`{"foo": "bar"`"#)]
    // variables
    #[case("variable_ref", "$foo")]

    fn it_recognizes_token(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, get_token(input).kind);
    }

    #[rstest]
    #[case("quoted_string", r#""\\""#)]
    fn quoted_string(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, get_token(input).kind);
    }

    #[rstest]
    #[case("unquoted_string", " foo")]
    #[case("unquoted_string", "\u{8}foo")]
    #[case("unquoted_string", "\nfoo")]
    #[case("unquoted_string", "\u{b}foo")]
    #[case("unquoted_string", "\rfoo")]
    #[case("unquoted_string", "\tfoo")]
    fn it_skips_whitespace(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, get_token(input).kind);
    }

    #[rstest]
    #[case("let", "let")]
    #[case("in", "in")]
    fn it_tokenizes_let_expression_keywords(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, get_token(input).kind);
    }

    #[rstest]
    #[case("?")]
    fn it_fails(#[case] input: &str) {
        let result = tokenize(input).map_err(|e| e.kind);
        let expected = Err(Kind::Syntax);
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case("raw-string near ->'mismatched_quote <-", (1, 14), "'raw_string' 'mismatched_quote ")]
    #[case("quoted-string near ->\"mismatched_quote <-", (1, 17), "\"quoted_string\" \"mismatched_quote ")]
    #[case("JSON literal near ->`false <-", (1, 8), "`true` `false ")]
    fn raw_string(#[case] contains: &str, #[case] pos: (usize, usize), #[case] input: &str) {
        let result = tokenize(input);
        assert!(result.is_err());

        match result {
            Err(error) => {
                //assert!(error.message.contains(contains));
                assert_eq!(Kind::Syntax, error.kind);
                assert_eq!(Position::new(pos.0, pos.1), error.position.unwrap());
            }
            _ => unreachable!(),
        }
    }

    fn get_token(input: &str) -> Rc<Lexeme> {
        return get_token_at(input, 0);
    }
    fn get_token_at(input: &str, index: usize) -> Rc<Lexeme> {
        return get_tokens(input)[index].clone();
    }
    fn get_tokens(input: &str) -> Vec<Rc<Lexeme>> {
        return tokenize(input).unwrap();
    }
}
