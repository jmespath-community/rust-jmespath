//! JMESPath Community is [the community implementation](https://jmespath.site/) of JMESPath,
//! a query and transformation language for JSON.
//!
//! # Evaluating JMESPath Expression
//!
//! Use the [search](crate::search) function to evaluate a JMESPath expression.
//!
//! ## Example
//! ```rust
//! use jmespath_community as jmespath;
//! use jmespath::{search, Value};
//!
//!// Parse some JSON data into a JMESPath variable
//!let json_str = "{\"foo\":{\"bar\":{\"baz\":true}}}";
//!let data = Value::from_json(json_str).unwrap();
//!
//!let result = search("foo.bar | baz", &data).unwrap();
//!assert_eq!(true, result);
//! ```
//!
//! A JMESPath expression can be parsed once and evaluated
//! multiple times using the [parse](crate::parser::parse) function.
//!
//! ## Example
//! ```rust
//! use jmespath_community as jmespath;
//! use jmespath::{parse, Value};
//!
//! let ast = parse("foo").unwrap();
//! let data = Value::from_json(r#"{"foo": "bar"}"#).unwrap();
//! let result = ast.search(&data).unwrap();
//! assert_eq!("bar", result);
//! ```
//!
//! # Registering Custom Functions
//!
//! JMESPath Community comes with a host of useful [builtin functions](https://jmespath.site/main/#functions).
//! However, it can be extended with [third-party functions](crate::function).
//!
//! ## Example
//! ```rust
//! mod custom_functions {
//!
//!		use jmespath_community as jmespath;
//!
//!     use jmespath::function;
//!
//!     use jmespath::errors::Error as RuntimeError;
//!
//!     use jmespath::FunctionContext;
//!     use jmespath::Value;
//!
//!     use jmespath::functions::ReturnValue;
//!     use jmespath::functions::Function;
//!
//!     use jmespath::functions::DataType;
//!     use jmespath::functions::ParamTypes::*;
//!     use jmespath::functions::Parameter;
//!     use jmespath::functions::Parameter::*;
//!
//!     function!(
//!         add,
//!         [
//!             left => Required(Of(DataType::Number)),
//!             right => Required(Of(DataType::Number))
//!         ],
//!         arguments,
//!         {
//!             // type checking has been performed by the runtime
//!             // safe to unwrap
//!
//!             let i = arguments[0].as_f64().unwrap();
//!             let j = arguments[1].as_f64().unwrap();
//!
//!             Value::from_f64(i + j)
//!         }
//!     );
//! }
//! ```
//!
//! Create a new instance of the JMESPath [Runtime](crate::Runtime) object and
//! register your custom function:
//!
//! ## Example
//! ```compile_fail
//! use jmespath_community as jmespath;
//! use jmespath::FunctionRegistrar;
//! use jmespath::{Runtime, Value};
//!
//! let add = Box::new(custom_functions::add::new());
//! let mut runtime = Runtime::create_runtime();
//! runtime.register(add);
//!
//! let expression = "foo";
//! let root = Value::Null;
//! let result = runtime.search(expression, &root).unwrap();
//!
//! assert_eq!(None, result);
//! ```
mod api;
mod lexer;
mod parser;
mod registry;
mod scopes;
mod utils;

/// Contains the types supporting error handling for this crate.
pub mod errors;
/// Defines the builtin JMESPath function implementations and
/// various helpers for authoring custom third-party functions.
pub mod functions;
/// Contains the main JMESPath expression interpreter.
pub(crate) mod interpreter;

pub(crate) mod runtime;

pub(crate) mod value;
pub(crate) mod value_eq;
pub(crate) mod value_from;
pub(crate) mod value_option;

/// A type that represents a JMESPath function that can be stored
/// into a thread-safe registry.
pub type JmesPathFunction = dyn crate::functions::Function + Send + Sync;

pub use api::*;

pub use utils::map::Map;
pub use utils::Number;

pub use errors::Error;
pub use parser::parse;
pub use parser::NodeType;
pub use parser::Slice;
pub use parser::AST;
pub use runtime::ByFunctionHolder;
pub use runtime::FunctionContext;
pub use runtime::FunctionRegistrar;
pub use runtime::Runtime;
pub use value::Value;
