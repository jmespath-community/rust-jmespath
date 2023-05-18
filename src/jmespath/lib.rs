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
pub mod interpreter;

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
