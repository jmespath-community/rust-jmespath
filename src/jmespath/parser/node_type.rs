use super::AST;
use crate::Map;

/// Represents the contents of an abstract syntax tree node.
#[derive(Clone, Debug)]
pub enum NodeType {
    /// A meaningless placeholder node.
    None,

    /// An unvalidated JSON literal string. _e.g_ `` `true` ``  or `` `{"foo": "bar"}` ``
    ///
    /// Note that the parser does not fully validate the syntax for a quoted-string
    /// for performance and simplicity. Instead, a syntax error will be
    /// reported during evaluation.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("`null`").unwrap();
    /// assert_eq!("(1, 1):JsonValue(\"null\")", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::JsonValue(..)));
    JsonValue(String),
    /// A positive or negative number. _e.g_ `42`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[42]").unwrap();
    /// assert_eq!("(1, 1):IndexExpression([(0, 0):None, (1, 2):Number(42)])", format!("{:?}", ast));
    Number(i32),
    /// An unvalidated identifier expressed as a quoted-string. _e.g_ `"foo"`.
    ///
    /// Note that the parser does not fully validate the syntax for a quoted-string
    /// for performance and simplicity. Instead, a syntax error will be
    /// reported during evaluation.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse(r#""foo""#).unwrap();
    /// assert_eq!("(1, 1):QuotedIdentifier(\"\\\"foo\\\"\")", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::QuotedIdentifier(..)));
    QuotedIdentifier(String),
    /// A raw string literal. _e.g_ `'text'`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("'foo'").unwrap();
    /// assert_eq!("(1, 1):RawString(\"foo\")", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::RawString(..)));
    RawString(String),
    /// An identifier expressed as an unquoted-string. _e.g_ `foo`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo").unwrap();
    /// assert_eq!("(1, 1):UnquotedIdentifier(\"foo\")", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::UnquotedIdentifier(..)));
    UnquotedIdentifier(String),
    /// A variable reference. _e.g_ `$foo`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("$foo").unwrap();
    /// assert_eq!("(1, 1):VariableRef(\"$foo\")", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::VariableRef(..)));
    VariableRef(String),

    /// A reference `$` to the root (input) value.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("$").unwrap();
    /// assert_eq!("(1, 1):RootNode", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::RootNode));
    RootNode,
    /// A reference `@` to the current node.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("@").unwrap();
    /// assert_eq!("(1, 1):CurrentNode", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::CurrentNode));
    CurrentNode,

    /// The filter `[?]` comparator [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[?`true`]").unwrap();
    /// assert_eq!("(1, 1):Projection([(1, 1):Filter([(1, 3):JsonValue(\"true\")]), (0, 0):None, (0, 0):None])", format!("{:?}", ast));
    /// assert!(matches!(ast.node_type, NodeType::Projection(..)));
    ///
    /// let ast = jmespath::parse("foo[?`true`]").unwrap();
    /// assert_eq!("(1, 4):Projection([(1, 4):Filter([(1, 6):JsonValue(\"true\")]), (1, 1):UnquotedIdentifier(\"foo\"), (0, 0):None])", format!("{:?}", ast));
    ///
    /// let ast = jmespath::parse("foo[?`true`].bar").unwrap();
    /// assert_eq!("(1, 4):Projection([(1, 4):Filter([(1, 6):JsonValue(\"true\")]), (1, 1):UnquotedIdentifier(\"foo\"), (1, 14):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    Filter(Vec<AST>),
    /// The flatten `[]` [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[]").unwrap();
    /// assert_eq!("(1, 1):Projection([(1, 1):Flatten, (0, 0):None, (0, 0):None])", format!("{:?}", ast));
    Flatten,
    /// The list wildcard `[*]` [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[*]").unwrap();
    /// assert_eq!("(1, 1):Projection([(1, 1):ListWildcard, (0, 0):None, (0, 0):None])", format!("{:?}", ast));
    ListWildcard,
    /// The slice `[::]` [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[::-1]").unwrap();
    /// assert_eq!("(1, 1):Projection([(1, 1):Slice(Slice { start: None, stop: None, step: Some(-1) }), (0, 0):None, (0, 0):None])", format!("{:?}", ast));
    Slice(Slice),

    /// The plus `+` arithmetic operator.
    /// # Example
    /// See [`NodeType::ArithmeticExpression`].
    Plus,
    /// The minus `−` (U+2212 MINUS SIGN) or `-` arithmetic operator.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("− bar").unwrap();
    /// assert_eq!("(1, 1):ArithmeticExpression([(0, 0):None, (1, 1):Minus, (1, 3):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    Minus,
    /// The multiplication `×` (U+00D7 MUTIPLY SIGN) arithmetic operator.
    ///
    /// Note: the `*` (star) character can also be used as a legitimate multiply sign.
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo × bar").unwrap();
    /// assert_eq!("(1, 5):ArithmeticExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):Multiply, (1, 7):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    Multiply,
    /// The divide `÷` (U+00F7 DIVISION SIGN) or `/` arithmetic operator.
    ///
    /// See [`NodeType::ArithmeticExpression`].
    Divide,
    ///The modulo `%` arithmetic operator.
    ///
    /// See [`NodeType::ArithmeticExpression`].
    Modulo,
    /// The integer division `//` arithmetic operator.
    ///
    /// See [`NodeType::ArithmeticExpression`].
    Div,

    /// The equal `==` logical comparator.
    ///
    /// See [`NodeType::ComparatorExpression`].
    Equal,
    /// The greater-than `>` logical comparator.
    /// # Example
    /// See [`NodeType::ComparatorExpression`].
    GreaterThan,
    /// The greater-than-or-equal `>=` logical comparator.
    ///
    /// See [`NodeType::ComparatorExpression`].
    GreaterThanOrEqual,
    /// The less-than `<` logical comparator.
    ///
    /// See [`NodeType::ComparatorExpression`].
    LessThan,
    /// The less-than-or-equal `<=` logical comparator.
    ///
    /// See [`NodeType::ComparatorExpression`].
    LessThanOrEqual,
    /// The not-equal `!=` logical comparator.
    ///
    /// See [`NodeType::ComparatorExpression`].
    NotEqual,

    /// The AND `&&` binary logical operator.
    ///
    /// See [`NodeType::LogicalExpression`].
    And,
    /// The OR `&&` binary logical operator.
    /// # Example
    /// See [`NodeType::LogicalExpression`].
    Or,
    /// The NOT `!` unary logical operator.
    ///
    /// See [`NodeType::LogicalExpression`].
    Not,

    /// An expression-type: `&<expression>`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("min_by(foo, &age)").unwrap();
    /// assert_eq!("(1, 1):FunctionExpression([(1, 1):UnquotedIdentifier(\"min_by\"), (1, 8):FunctionArguments([(1, 8):UnquotedIdentifier(\"foo\"), (1, 13):Expression([(1, 14):UnquotedIdentifier(\"age\")])])])", format!("{:?}", ast));
    Expression(Vec<AST>),

    /// A paren-expression `( <expression> )`.
    ParenExpression(Vec<AST>),

    /// A binary pipe-expression `foo | bar`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo | bar").unwrap();
    /// assert_eq!("(1, 5):PipeExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 7):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    PipeExpression(Vec<AST>),
    /// A binary sub-expression `foo.bar`.
    ///
    /// Note that to simplify evaluation of a JMESPath expression
    /// most sub-expressions on the right-hand-side of a projection
    /// expressions are represented in terms of a [`NodeType::Projection`].
    ///
    /// For instance, `` foo.bar `` is a [`NodeType::SubExpression`].
    /// Whereas, `` foo[*].bar ``, while technically a sub-expression as per
    /// the grammar rules, is handled as a [`NodeType::Projection`] instead.
    ///
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo.bar").unwrap();
    /// assert_eq!("(1, 4):SubExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    SubExpression(Vec<AST>),

    /// Supports the [`NodeType::Filter`], [`NodeType::Flatten`] , [`NodeType::ListWildcard`] and [`NodeType::Slice`] projections.
    ///
    /// A projection has three nodes:
    /// - [`NodeType::Filter`], [`NodeType::Flatten`], [`NodeType::ListWildcard`] or [`NodeType::Slice`].
    /// - An optional left node.
    /// - An optional right node.
    /// # Example
    /// See [`NodeType::Filter`].
    Projection(Vec<AST>),
    /// The hash wildcard `*` [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("*").unwrap();
    /// assert_eq!("(1, 1):HashWildcardProjection([(0, 0):None, (0, 0):None])", format!("{:?}", ast));
    HashWildcardProjection(Vec<AST>),

    /// A binary arithmetic-expression `left { + | − | - | × | * | ÷ | / | % | // } right`  
    /// or unary arithmetic expression `{ + | − | - } right`.
    ///
    /// Arithmetic expressions support the four basic operations, as well as the modulo and integer division.
    /// Use the following arithmetic operators:
    /// - The minus `+` addition operator.
    /// - The minus `−` (U+2212 MINUS SIGN) or `-` subtraction operator.
    /// - The multiply `×` (U+00D7 MUTIPLY SIGN) arithmetic operator.
    /// - The divide `÷` (U+00F7 DIVISION SIGN) or `/` arithmetic operator.
    /// - The modulo `%` operator. _e.g_ ` 7 % 3 ` -> `1`.
    /// - The integer division `//` operator. _e.g_ ` 7 // 3 ` –> `2`.
    ///
    /// Arithmetic expressions also support the unary negative or positive expressions:
    ///
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo + bar").unwrap();
    /// assert_eq!("(1, 5):ArithmeticExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):Plus, (1, 7):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    ArithmeticExpression(Vec<AST>),

    /// A binary comparator-expression `left { < | <= | = | != | >= | > } right`.
    ///
    /// A comparator-expression is used in the filter [`NodeType::Filter`] [`NodeType::Projection`].
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo > `2`").unwrap();
    /// assert_eq!("(1, 5):ComparatorExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):GreaterThan, (1, 7):JsonValue(\"2\")])", format!("{:?}", ast));
    ComparatorExpression(Vec<AST>),

    /// A function expression `avg(foo[*])`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("length(@)").unwrap();
    /// assert_eq!("(1, 1):FunctionExpression([(1, 1):UnquotedIdentifier(\"length\"), (1, 8):FunctionArguments([(1, 8):CurrentNode])])", format!("{:?}", ast));
    /// ```
    ///
    /// See also [`NodeType::Expression`].
    FunctionExpression(Vec<AST>),
    /// Supports the [`NodeType::FunctionExpression`] variant.
    /// # Example
    /// See [`NodeType::FunctionArguments`].
    FunctionArguments(Vec<AST>),

    /// An index-expression `[0]` or `foo[0]`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo[42]").unwrap();
    /// assert_eq!("(1, 4):IndexExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):Number(42)])", format!("{:?}", ast));
    IndexExpression(Vec<AST>),
    /// Supports the [`NodeType::IndexExpression`] AST node.
    #[doc(hidden)]
    Index(Vec<AST>),

    /// A let expression `let $foo = bar in baz`.
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("let $foo = 'bar' in baz").unwrap();
    /// assert_eq!("(1, 1):LetExpression([(1, 10):LetBindings([(1, 5):VariableRef(\"$foo\"), (1, 12):RawString(\"bar\")]), (1, 21):UnquotedIdentifier(\"baz\")])", format!("{:?}", ast));
    LetExpression(Vec<AST>),
    /// Supports the [`NodeType::LetExpression`] AST node.
    LetBindings(Vec<AST>),

    /// A unary or binary logical-expression `left { && | || } right`  
    /// or unary negative predicate `! foo`.
    ///
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("foo || bar").unwrap();
    /// assert_eq!("(1, 5):LogicalExpression([(1, 1):UnquotedIdentifier(\"foo\"), (1, 5):Or, (1, 8):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    LogicalExpression(Vec<AST>),

    /// A multi-select-hash `{foo:foo, bar:bar}`
    ///
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("{foo: foo}").unwrap();
    /// assert_eq!("(1, 1):MultiSelectHash({\"foo\": (1, 7):UnquotedIdentifier(\"foo\")})", format!("{:?}", ast));
    MultiSelectHash(Map<String, AST>),

    /// A multi-select-list `[foo, bar]`
    ///
    /// # Example
    /// ```
    /// use jmespath::{AST, NodeType};
    /// let ast = jmespath::parse("[foo, bar]").unwrap();
    /// assert_eq!("(1, 1):MultiSelectList([(1, 2):UnquotedIdentifier(\"foo\"), (1, 7):UnquotedIdentifier(\"bar\")])", format!("{:?}", ast));
    MultiSelectList(Vec<AST>),
}

macro_rules! as_ {
    ($ident:ident, $enum:ident) => {
        pub fn $ident(&self) -> Option<&Vec<AST>> {
            match self {
                Self::$enum(v) => Some(v),
                _ => None,
            }
        }
    };
}
impl NodeType {
    as_!(as_index, Index);
}

/// Represents the parameters for a slice [`NodeType::Projection`].
#[derive(Debug, Clone)]
pub struct Slice {
    pub start: Option<isize>,
    pub stop: Option<isize>,
    pub step: Option<isize>,
}
