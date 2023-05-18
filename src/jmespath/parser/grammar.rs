use santiago::{
    grammar::{Associativity, Grammar},
    lexer::Lexeme,
};

use super::{node_type::Slice, NodeType, AST};
use crate::{errors::Position, map, Map};

pub fn grammar() -> Grammar<AST> {
    santiago::grammar!(

        // grammar

        "expression" => rules "current";
        "expression" => rules "identifier";
        "expression" => rules "json_value";
        "expression" => rules "raw_string";
        "expression" => rules "root";
        "expression" => rules "variable_ref";

        "expression" => rules "let" "bindings" "in" "expression" => let_expression;
        "expression" => rules "lparen" "expression" "rparen" => paren_expression;
        "expression" => rules "expression" "pipe" "expression" => pipe_expression;

        // sub-expression

        "sub_expression_rhs" => rules "function_expression";
        "sub_expression_rhs" => rules "hash_wildcard_projection";
        "sub_expression_rhs" => rules "identifier";
        "sub_expression_rhs" => rules "multi_select_hash";
        "sub_expression_rhs" => rules "multi_select_list";

        "expression" => rules "expression" "dot" "sub_expression_rhs" => sub_expression;

        // arithmetic-expression

        "expression" => rules "expression" "div" "expression" => arithmetic_expression;
        "expression" => rules "expression" "divide" "expression" => arithmetic_expression;
        "expression" => rules "expression" "minus" "expression" => arithmetic_expression;
        "expression" => rules "expression" "mod" "expression" => arithmetic_expression;
        "expression" => rules "expression" "multiply" "expression" => arithmetic_expression;
        "expression" => rules "expression" "plus" "expression" => arithmetic_expression;
        "expression" => rules "expression" "star" "expression" => arithmetic_expression;

        "unary_arithmetic_expression" => rules "plus" "expression" => arithmetic_expression;
        "unary_arithmetic_expression" => rules "minus" "expression" => arithmetic_expression;

        "expression" => rules "unary_arithmetic_expression";

        // comparator-expression

        "expression" => rules "expression" "equal" "expression" => comparator_expression;
        "expression" => rules "expression" "greater_than_or_equal" "expression" => comparator_expression;
        "expression" => rules "expression" "greater_than" "expression" => comparator_expression;
        "expression" => rules "expression" "less_than_or_equal" "expression" => comparator_expression;
        "expression" => rules "expression" "less_than" "expression" => comparator_expression;
        "expression" => rules "expression" "not_equal" "expression" => comparator_expression;

        // filter projection

        "filter_specifier" => rules "filter" "expression" "rbracket" => filter;

        "expression" => rules "filter_specifier" => projection;
        "expression" => rules "expression" "filter_specifier" => projection;

        // flatten projection

        "flatten_specifier" => rules "flatten" => flatten;

        "expression" => rules "flatten_specifier" => projection;
        "expression" => rules "expression" "flatten_specifier" => projection;

        // function-expression

        "expression_type" => rules "expref" "expression" => expression_type;

        "function_argument_list" => rules "expression" => function_argument_list;
        "function_argument_list" => rules "expression_type" => function_argument_list;
        "function_argument_list" => rules "function_argument_list" "comma" "expression" => function_argument_list;
        "function_argument_list" => rules "function_argument_list" "comma" "expression_type" => function_argument_list;

        "function_arguments" => rules "lparen" "rparen" => function_arguments;
        "function_arguments" => rules "lparen" "function_argument_list" "rparen" => function_arguments;

        "function_expression" => rules "unquoted_string" "function_arguments" => function_expression;

        "expression" => rules "function_expression";

        // hash-wildcard projection

        "hash_wildcard_projection" => rules "star" => hash_wildcard;

        "expression" => rules "hash_wildcard_projection";

        // index_expression

        "index" => rules "lbracket" "number" "rbracket" => index;

        "expression" => rules "index" => index_expression;
        "expression" => rules "expression" "index" => index_expression;

        // let-expression

        "bindings" => rules "binding" => let_bindings;
        "bindings" => rules "bindings" "comma" "binding" => let_bindings;
        "binding" => rules "variable_ref" "assign" "expression" => let_binding;

        "expression" => rules "let" "bindings" "in" "expression" => let_expression;

        // list-wilcard projection

        "list_wildcard_specifier" => rules "lbracket" "star" "rbracket" => list_wildcard;

        "expression" => rules "list_wildcard_specifier" => projection;
        "expression" => rules "expression" "list_wildcard_specifier" => projection;

        // logical-expression

        "expression" => rules "expression" "and" "expression" => logical_expression;
        "expression" => rules "expression" "or" "expression" => logical_expression;
        "expression" => rules "expression" "not" "expression" => logical_expression;

        "unary_logical_expression" => rules "not" "expression" => logical_expression;

        "expression" => rules "unary_logical_expression";

        //multi-select-hash

        "multi_select_hash_key_values" => rules "identifier" "colon" "expression" => multi_select_hash_key_value;
        "multi_select_hash_key_values" => rules "multi_select_hash_key_values" "comma" "multi_select_hash_key_values" => multi_select_hash_key_values;

        "multi_select_hash" => rules "lbrace" "multi_select_hash_key_values" "rbrace" => multi_select_hash;

        "expression" => rules "multi_select_hash";

        //multi-select-list

        "multi_select_expressions" => rules "expression" => multi_select_list_expression;
        "multi_select_expressions" => rules "multi_select_expressions" "comma" "multi_select_expressions" => multi_select_list_expressions;

        "multi_select_list" => rules "lbracket" "multi_select_expressions" "rbracket" => multi_select_list;

        "expression" => rules "multi_select_list";

        // paren-expression

        "expression" => rules "lparen" "expression" "rparen" => paren_expression;

        // pipe-expression

        "expression" => rules "expression" "pipe" "expression" => pipe_expression;

        // slice projection

        "slice" => rules "lbracket"          "colon"                           "rbracket" => |nodes| slice_bracket(None, None, None, nodes[0].position);
        "slice" => rules "lbracket" "number" "colon"                           "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), None, None, nodes[0].position);
        "slice" => rules "lbracket"          "colon" "number"                  "rbracket" => |nodes| slice_bracket(None, Some(nodes[2].number()), None, nodes[0].position);
        "slice" => rules "lbracket" "number" "colon" "number"                  "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), Some(nodes[3].number()), None, nodes[0].position);

        "slice" => rules "lbracket"          "colon"          "colon"          "rbracket" => |nodes| slice_bracket(None, None, None, nodes[0].position);
        "slice" => rules "lbracket" "number" "colon"          "colon"          "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), None, None, nodes[0].position);
        "slice" => rules "lbracket"          "colon" "number" "colon"          "rbracket" => |nodes| slice_bracket(None, Some(nodes[2].number()), None, nodes[0].position);
        "slice" => rules "lbracket" "number" "colon" "number" "colon"          "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), Some(nodes[3].number()), None, nodes[0].position);

        "slice" => rules "lbracket"          "colon"          "colon" "number" "rbracket" => |nodes| slice_bracket(None, None, Some(nodes[3].number()), nodes[0].position);
        "slice" => rules "lbracket" "number" "colon"          "colon" "number" "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), None, Some(nodes[4].number()), nodes[0].position);
        "slice" => rules "lbracket"          "colon" "number" "colon" "number" "rbracket" => |nodes| slice_bracket(None, Some(nodes[2].number()), Some(nodes[4].number()), nodes[0].position);
        "slice" => rules "lbracket" "number" "colon" "number" "colon" "number" "rbracket" => |nodes| slice_bracket(Some(nodes[1].number()), Some(nodes[3].number()), Some(nodes[5].number()), nodes[0].position);

        "expression" => rules "slice" => projection;
        "expression" => rules "expression" "slice" => projection;

        // other lexemes

        "identifier" => lexemes "quoted_string"        => quoted_string;
        "identifier" => lexemes "unquoted_string"      => unquoted_string;

        "current" => lexemes "current"                 => current_node;
        "json_value" => lexemes "json_value"           => json_value;
        "number" => lexemes "number"                   => number;
        "quoted_string" => lexemes "quoted_string"     => quoted_string;
        "raw_string" => lexemes "raw_string"           => raw_string;
        "root" => lexemes "root"                       => root_node;
        "unquoted_string" => lexemes "unquoted_string" => unquoted_string;
        "variable_ref" => lexemes "variable_ref"       => variable_ref;

        "div" => lexemes "div"           => arithmetic_div;
        "divide" => lexemes "divide"     => arithmetic_divide;
        "minus" => lexemes "minus"       => arithmetic_minus;
        "mod" => lexemes "mod"           => arithmetic_modulo;
        "multiply" => lexemes "multiply" => arithmetic_multiply;
        "plus" => lexemes "plus"         => arithmetic_plus;
        "star" => lexemes "star"         => arithmetic_multiply;

        "and" => lexemes "and"           => logical_and;
        "or" => lexemes "or"             => logical_or;
        "not" => lexemes "not"           => logical_not;

        "equal" => lexemes "equal"                                 => comparator_equal;
        "greater_than_or_equal" => lexemes "greater_than_or_equal" => comparator_greater_than_or_equal;
        "greater_than" => lexemes "greater_than"                   => comparator_greater_than;
        "less_than_or_equal" => lexemes "less_than_or_equal"       => comparator_less_than_or_equal;
        "less_than" => lexemes "less_than"                         => comparator_less_than;
        "not_equal" => lexemes "not_equal"                         => comparator_not_equal;

        "assign" => lexemes "assign"     => ignored;
        "colon" => lexemes "colon"       => ignored;
        "comma" => lexemes "comma"       => ignored;
        "dot" => lexemes "dot"           => ignored;
        "expref" => lexemes "expref"     => ignored;
        "filter" => lexemes "filter"     => ignored;
        "flatten" => lexemes "flatten"   => ignored;
        "in" => lexemes "in"             => ignored;
        "lbrace" => lexemes "lbrace"     => ignored;
        "lbracket" => lexemes "lbracket" => ignored;
        "let" => lexemes "let"           => ignored;
        "list" => lexemes "list"         => ignored;
        "lparen" => lexemes "lparen"     => ignored;
        "pipe" => lexemes "pipe"         => ignored;
        "rbrace" => lexemes "rbrace"     => ignored;
        "rbracket" => lexemes "rbracket" => ignored;
        "rparen" => lexemes "rparen"     => ignored;

        Associativity::Left => rules "lbracket";
        Associativity::Left => rules "star";
        Associativity::Left => rules "flatten";
        Associativity::Left => rules "filter";
        Associativity::Left => rules "rbracket";

        Associativity::Left => rules "assign";

        Associativity::Left => rules "pipe";

        Associativity::Left => rules "not";
        Associativity::Left => rules "or";
        Associativity::Left => rules "and";

        Associativity::Left => rules "equal";
        Associativity::Left => rules "greater_than";
        Associativity::Left => rules "greater_than_or_equal";
        Associativity::Left => rules "less_than";
        Associativity::Left => rules "less_than_or_equal";
        Associativity::Left => rules "not_equal";

        Associativity::Left => rules "plus";
        Associativity::Left => rules "minus";
        Associativity::Left => rules "multiply";
        Associativity::Left => rules "divide";
        Associativity::Left => rules "mod";
        Associativity::Left => rules "div";

        Associativity::Left => rules "dot";
    )
}

macro_rules! pos(
    ($ident:ident) => {
        Position::new($ident[0].position.line, $ident[0].position.column)
    };
);

fn ignored<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::None, pos!(lexemes))
}

fn arithmetic_div<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Div, pos!(lexemes))
}
fn arithmetic_divide<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Divide, pos!(lexemes))
}
fn arithmetic_modulo<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Modulo, pos!(lexemes))
}
fn arithmetic_multiply<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Multiply, pos!(lexemes))
}
fn arithmetic_plus<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Plus, pos!(lexemes))
}
fn arithmetic_minus<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Minus, pos!(lexemes))
}
fn comparator_equal<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Equal, pos!(lexemes))
}
fn comparator_greater_than<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::GreaterThan, pos!(lexemes))
}
fn comparator_greater_than_or_equal<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::GreaterThanOrEqual, pos!(lexemes))
}
fn comparator_less_than<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::LessThan, pos!(lexemes))
}
fn comparator_less_than_or_equal<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::LessThanOrEqual, pos!(lexemes))
}
fn comparator_not_equal<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::NotEqual, pos!(lexemes))
}
fn current_node<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::CurrentNode, pos!(lexemes))
}
fn json_value<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let raw = &lexemes[0].raw;
    let text = &raw[1..raw.len() - 1].replace(r#"\`"#, "`");

    AST::make(NodeType::JsonValue(text.to_string()), pos!(lexemes))
}
fn logical_and<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::And, pos!(lexemes))
}
fn logical_or<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Or, pos!(lexemes))
}
fn logical_not<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::Not, pos!(lexemes))
}
fn number<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let text = &lexemes[0].raw;
    let number = text.parse::<i32>().unwrap();
    AST::make(NodeType::Number(number), pos!(lexemes))
}
fn quoted_string<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let raw = &lexemes[0].raw;
    AST::make(NodeType::QuotedIdentifier(raw.to_string()), pos!(lexemes))
}
fn raw_string<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let raw = &lexemes[0].raw;
    let text = &raw[1..raw.len() - 1]
        .replace(r#"\'"#, r#"'"#)
        .replace(r#"\\"#, r#"\"#);
    AST::make(NodeType::RawString(text.to_string()), pos!(lexemes))
}
fn root_node<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    AST::make(NodeType::RootNode, pos!(lexemes))
}
fn unquoted_string<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let identifier = &lexemes[0].raw;
    AST::make(
        NodeType::UnquotedIdentifier(identifier.to_string()),
        pos!(lexemes),
    )
}
fn variable_ref<'a, 'b>(lexemes: &'a [&'b std::rc::Rc<Lexeme>]) -> AST {
    let text = &lexemes[0].raw;
    AST::make(NodeType::VariableRef(text.to_string()), pos!(lexemes))
}

fn arithmetic_expression(nodes: Vec<AST>) -> AST {
    if nodes.len() == 2 {
        return AST::make(
            NodeType::ArithmeticExpression(vec![
                AST::make(NodeType::None, Position::new(0, 0)),
                nodes[0].clone(),
                nodes[1].clone(),
            ]),
            nodes[0].position,
        );
    }
    AST::make(
        NodeType::ArithmeticExpression(vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()]),
        nodes[1].position,
    )
}
fn comparator_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::ComparatorExpression(vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()]),
        nodes[1].position,
    )
}
fn expression_type(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::Expression(vec![nodes[1].clone()]),
        nodes[0].position,
    )
}
fn flatten(nodes: Vec<AST>) -> AST {
    AST::make(NodeType::Flatten, nodes[0].position)
}
fn filter(nodes: Vec<AST>) -> AST {
    AST::make(NodeType::Filter(vec![nodes[1].clone()]), nodes[0].position)
}
fn function_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::FunctionExpression(nodes.clone()),
        nodes[0].position,
    )
}
fn function_argument_list(nodes: Vec<AST>) -> AST {
    match nodes.len() {
        1 => nodes[0].clone(),
        3 => {
            let mut args = Vec::new();
            match &nodes[0] {
                AST {
                    node_type: NodeType::FunctionArguments(list),
                    ..
                } => {
                    args.extend(list.clone());
                }
                _ => {
                    args.push(nodes[0].clone());
                }
            }
            args.push(nodes[2].clone());
            AST::make(NodeType::FunctionArguments(args), nodes[0].position)
        }
        _ => unreachable!(),
    }
}
fn function_arguments(nodes: Vec<AST>) -> AST {
    if nodes.len() == 3 {
        return match &nodes[1] {
            AST {
                node_type: NodeType::FunctionArguments(..),
                ..
            } => nodes[1].clone(),
            _ => AST::make(
                NodeType::FunctionArguments(vec![nodes[1].clone()]),
                nodes[1].position,
            ),
        };
    }
    AST::make(NodeType::FunctionArguments(vec![]), nodes[0].position)
}
fn make_hash_wildcard_projection(hash: &AST, inner: &AST) -> AST {
    match &hash.node_type {
        NodeType::HashWildcardProjection(list) => {
            let mut vec = list.clone();
            vec[1] = match &list[1].node_type {
                NodeType::HashWildcardProjection(..) => {
                    make_hash_wildcard_projection(&list[1], inner)
                }
                _ => inner.clone(),
            };
            AST::make(NodeType::HashWildcardProjection(vec), list[0].position)
        }
        _ => unreachable!(),
    }
}
fn hash_wildcard(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::HashWildcardProjection(vec![
            AST::make(NodeType::None, Position::new(0, 0)),
            AST::make(NodeType::None, Position::new(0, 0)),
        ]),
        nodes[0].position,
    )
}
fn index(nodes: Vec<AST>) -> AST {
    AST::make(NodeType::Index(vec![nodes[1].clone()]), nodes[0].position)
}
fn make_projection_index_expression(projection: &AST, inner: &AST) -> AST {
    let children = match &projection.node_type {
        NodeType::Projection(vec) => {
            let mut children = vec.clone();
            children[2] = match &vec[2].node_type {
                NodeType::HashWildcardProjection(..) => {
                    make_hash_wildcard_projection(&vec[2], inner)
                }
                _ => inner.clone(),
            };
            children
        }
        _ => unreachable!(),
    };
    AST::make(NodeType::Projection(children), projection.position)
}
fn index_expression(nodes: Vec<AST>) -> AST {
    let mut children = Vec::new();
    let mut position = nodes[0].position;
    match &nodes[0].node_type {
        NodeType::LetExpression(children) => {
            // let-expression needs refactoring (see fn projection)
            // here we change `` let-expression [bindings, <expression>] <index> ``
            // to `` let-expression [bindings, <expression><indenx>] ``
            let expression = index_expression(vec![children[1].clone(), nodes[1].clone()]);
            return AST::make(
                NodeType::LetExpression(vec![children[0].clone(), expression]),
                nodes[0].position,
            );
        }
        NodeType::Projection(children) => {
            return make_projection_index_expression(
                &nodes[0],
                &index_expression(vec![children[2].clone(), nodes[1].clone()]),
            )
        }
        NodeType::HashWildcardProjection(children) => {
            return make_hash_wildcard_projection(
                &nodes[0],
                &index_expression(vec![children[1].clone(), nodes[1].clone()]),
            )
        }
        _ => {}
    }
    match &nodes[0].node_type {
        NodeType::Index(vec) => {
            children.push(AST::make(NodeType::None, Position::new(0, 0)));
            children.push(vec[0].clone());
        }
        _ => {
            children.push(nodes[0].clone());
            children.push(nodes[1].node_type.as_index().unwrap()[0].clone());
            position = nodes[1].position;
        }
    }
    AST::make(NodeType::IndexExpression(children), position)
}
fn let_binding(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::LetBindings(vec![nodes[0].clone(), nodes[2].clone()]),
        nodes[1].position,
    )
}
fn let_bindings(nodes: Vec<AST>) -> AST {
    match nodes.len() {
        1 => nodes[0].clone(),
        3 => match (&nodes[0].node_type, &nodes[2].node_type) {
            (NodeType::LetBindings(left), NodeType::LetBindings(right)) => {
                let mut bindings = left.clone();
                bindings.extend(right.clone());
                AST::make(NodeType::LetBindings(bindings), nodes[0].position)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
fn let_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::LetExpression(vec![nodes[1].clone(), nodes[3].clone()]),
        nodes[0].position,
    )
}
fn list_wildcard(nodes: Vec<AST>) -> AST {
    AST::make(NodeType::ListWildcard, nodes[0].position)
}
fn logical_expression(nodes: Vec<AST>) -> AST {
    if matches!(nodes[0].node_type, NodeType::Not) {
        return AST::make(
            NodeType::LogicalExpression(vec![nodes[0].clone(), nodes[1].clone()]),
            nodes[0].position,
        );
    }
    AST::make(
        NodeType::LogicalExpression(vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()]),
        nodes[1].position,
    )
}
fn multi_select_hash(nodes: Vec<AST>) -> AST {
    let mut hash = nodes[1].clone();
    hash.position = nodes[0].position;
    hash
}
fn multi_select_hash_key_value(nodes: Vec<AST>) -> AST {
    // "unquoted_identifier" "colon" "expression"
    let key = nodes[0].identifier().clone();
    let ast = nodes[2].clone();
    AST::make(
        NodeType::MultiSelectHash(map![key => ast]),
        nodes[1].position,
    )
}
fn multi_select_hash_key_values(nodes: Vec<AST>) -> AST {
    // "multi_select_hash_key_value" "comma" "multi_select_hash_keyvalue"
    let mut first = nodes[0].hashmap().clone();
    let next = nodes[2].hashmap();
    for item in next.iter() {
        first.insert(item.0.to_string(), item.1.clone());
    }
    AST::make(NodeType::MultiSelectHash(first), nodes[0].position)
}
fn multi_select_list(nodes: Vec<AST>) -> AST {
    let mut list = nodes[1].clone();
    list.position = nodes[0].position;
    list
}
fn multi_select_list_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::MultiSelectList(vec![nodes[0].clone()]),
        nodes[0].position,
    )
}
fn multi_select_list_expressions(nodes: Vec<AST>) -> AST {
    // create a flatten multi-select-list
    let mut args: Vec<AST> = vec![];
    match (&nodes[0].node_type, &nodes[2].node_type) {
        (NodeType::MultiSelectList(left), NodeType::MultiSelectList(right)) => {
            for item in left {
                args.push(item.clone());
            }
            for item in right {
                args.push(item.clone());
            }
        }
        _ => unreachable!(),
    }
    AST::make(NodeType::MultiSelectList(args), nodes[0].position)
}
fn paren_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::ParenExpression(vec![nodes[1].clone()]),
        nodes[0].position,
    )
}
fn pipe_expression(nodes: Vec<AST>) -> AST {
    AST::make(
        NodeType::PipeExpression(vec![nodes[0].clone(), nodes[2].clone()]),
        nodes[1].position,
    )
}
fn make_projection(projection_type: &AST, left: &AST, right: &AST) -> AST {
    let mut position = projection_type.position;
    let mut projection = vec![
        projection_type.clone(),
        left.clone(),
        AST::make(NodeType::None, Position::new(0, 0)),
    ];
    if matches!(projection_type.node_type, NodeType::Flatten) {
        return AST::make(NodeType::Projection(projection), position);
    }
    if matches!(left.node_type, NodeType::HashWildcardProjection(..)) {
        return make_hash_wildcard_projection(
            left,
            &make_projection(
                projection_type,
                right,
                &AST::make(NodeType::None, Position::new(0, 0)),
            ),
        );
    }
    match &left.node_type {
        NodeType::Projection(children) => match &children[0].node_type {
            _ => {
                position = children[0].position;
                projection = children.clone();
                projection[2] = make_projection(
                    projection_type,
                    &children[2],
                    &AST::make(NodeType::None, Position::default()),
                );
            }
        },
        _ => {
            projection[2] = right.clone();
        }
    }
    AST::make(NodeType::Projection(projection), position)
}
fn projection(nodes: Vec<AST>) -> AST {
    match &nodes[0].node_type {
        NodeType::Filter(..) | NodeType::Flatten | NodeType::ListWildcard | NodeType::Slice(..) => {
            make_projection(
                &nodes[0],
                &AST::make(NodeType::None, Position::new(0, 0)),
                &AST::make(NodeType::None, Position::new(0, 0)),
            )
        }
        NodeType::LetExpression(children) => {
            // I have not been able to successfully configure the parser for let-expression.
            // For instance the following expression yields two different parse trees:
            // `` let $foo = foo in foo[*] ``:
            // - `` (let $foo = foo in foo) [*] ``
            // - `` let $foo = foo in ( foo[*] ) ``
            //
            // Unfortunately, the disambiguation algorithm does not seem to work.
            let projection = vec![
                nodes[1].clone(),
                children[1].clone(),
                AST::make(NodeType::None, Position::new(0, 0)),
            ];
            let expression = AST::make(NodeType::Projection(projection), nodes[0].position);
            AST::make(
                NodeType::LetExpression(vec![children[0].clone(), expression]),
                nodes[0].position,
            )
        }
        NodeType::Projection(..) => make_projection(&nodes[1], &nodes[0], &nodes[1]),
        _ => make_projection(
            &nodes[1],
            &nodes[0],
            &AST::make(NodeType::None, Position::new(0, 0)),
        ),
    }
}
fn slice_bracket(
    start: Option<i32>,
    stop: Option<i32>,
    step: Option<i32>,
    position: Position,
) -> AST {
    AST::make(
        NodeType::Slice(Slice {
            start: start.map(|x| x as isize),
            stop: stop.map(|x| x as isize),
            step: step.map(|x| x as isize),
        }),
        position,
    )
}
fn make_sub_expression(nodes: Vec<AST>) -> AST {
    match &nodes[0].node_type {
        NodeType::HashWildcardProjection(children) => {
            let expression = match &children[1].node_type {
                NodeType::None => nodes[2].clone(),
                _ => make_sub_expression(vec![
                    children[1].clone(),
                    nodes[1].clone(),
                    nodes[2].clone(),
                ]),
            };
            AST::make(
                NodeType::HashWildcardProjection(vec![children[0].clone(), expression]),
                nodes[0].position,
            )
        }
        NodeType::LetExpression(children) => {
            let expression = make_sub_expression(vec![
                children[1].clone(),
                nodes[1].clone(),
                nodes[2].clone(),
            ]);
            AST::make(
                NodeType::LetExpression(vec![children[0].clone(), expression]),
                nodes[0].position,
            )
        }
        NodeType::Projection(children) => {
            let expression = match &children[2].node_type {
                NodeType::None => nodes[2].clone(),
                _ => make_sub_expression(vec![
                    children[2].clone(),
                    nodes[1].clone(),
                    nodes[2].clone(),
                ]),
            };
            AST::make(
                NodeType::Projection(vec![children[0].clone(), children[1].clone(), expression]),
                nodes[0].position,
            )
        }
        _ => AST::make(
            NodeType::SubExpression(vec![nodes[0].clone(), nodes[2].clone()]),
            nodes[1].position,
        ),
    }
}
fn sub_expression(nodes: Vec<AST>) -> AST {
    match (&nodes[0].node_type, &nodes[2].node_type) {
        (NodeType::LetExpression(children), _) => {
            // let-expression needs refactoring (see fn projection)
            // here we change: `let-expression [bindings, <lhs>] . <rhs> `
            // to: `let-expression [bindings, <lhs>.<rhs>]
            let expression = make_sub_expression(vec![
                children[1].clone(),
                nodes[1].clone(),
                nodes[2].clone(),
            ]);
            AST::make(
                NodeType::LetExpression(vec![children[0].clone(), expression]),
                nodes[0].position,
            )
        }
        (NodeType::HashWildcardProjection(..), NodeType::HashWildcardProjection(..)) => {
            make_sub_expression(nodes)
        }
        (_, NodeType::HashWildcardProjection(list)) => {
            let mut vec = list.clone();
            vec[0] = nodes[0].clone();
            AST::make(NodeType::HashWildcardProjection(vec), list[0].position)
        }
        _ => make_sub_expression(nodes),
    }
}
