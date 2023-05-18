use santiago::lexer::LexerRules;

pub fn lexer_rules() -> LexerRules {
    santiago::lexer_rules!(

        "DEFAULT" | "assign" = string "=";
        "DEFAULT" | "colon" = string ":";
        "DEFAULT" | "comma" = string ",";
        "DEFAULT" | "dot" = string ".";
        "DEFAULT" | "pipe" = string "|";

        "DEFAULT" | "lparen" = string "(";
        "DEFAULT" | "rparen" = string ")";
        "DEFAULT" | "lbrace" = string "{";
        "DEFAULT" | "rbrace" = string "}";
        "DEFAULT" | "lbracket" = string "[";
        "DEFAULT" | "rbracket" = string "]";

        "DEFAULT" | "filter" = string "[?";
        "DEFAULT" | "flatten" = string "[]";

        "DEFAULT" | "star" = string "*";
        "DEFAULT" | "current" = string "@";
        "DEFAULT" | "root" = string "$";
        "DEFAULT" | "expref" = string "&";

        // arithmetic operators
        "DEFAULT" | "plus" = string "+";
        "DEFAULT" | "minus" = pattern r"-|−";
        "DEFAULT" | "multiply" = string "×";
        "DEFAULT" | "divide" = pattern r"/|÷";
        "DEFAULT" | "mod" = string "%";
        "DEFAULT" | "div" = string "//";

        // comparison operators
        "DEFAULT" | "equal" = string "==";
        "DEFAULT" | "greater_than_or_equal" = string ">=";
        "DEFAULT" | "greater_than" = string ">";
        "DEFAULT" | "less_than_or_equal" = string "<=";
        "DEFAULT" | "less_than" = string "<";
        "DEFAULT" | "not_equal" = string "!=";

        // logical operators
        "DEFAULT" | "and" = string "&&";
        "DEFAULT" | "or" = string "||";
        "DEFAULT" | "not" = string "!";

        // number
        "DEFAULT" | "number" = pattern r"-?[0-9]+";

        // keywords - let expressions
        "DEFAULT" | "let" = string "let";
        "DEFAULT" | "in" = string "in";

        // identifiers
        "DEFAULT" | "quoted_string" = pattern r#""(\\([\\"/bfnrt]|u[0-9A-Fa-f]{4})|[^\\"])*""#;
        "DEFAULT" | "unquoted_string" = pattern r"[A-Za-z_][0-9A-Za-z_]*";

        // literals
        "DEFAULT" | "raw_string" = pattern r"'(\\[\\']|[^'])*'";
        "DEFAULT" | "json_value" = pattern r"`(\\`|[^`])+`";

        // bindings
        "DEFAULT" | "variable_ref" = pattern r"\$[A-Za-z_][0-9A-Za-z_]*";

        // Whitespace " " will be skipped
        "DEFAULT" | "WS" = pattern r"\s|\u{8}" => |lexer| lexer.skip();
    )
}
