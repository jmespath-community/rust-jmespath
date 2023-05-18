mod error;
mod kind;
mod position;
mod santiago;

pub(crate) mod error_builder;
pub(crate) mod invalid_arity;
pub(crate) mod invalid_type;
pub(crate) mod invalid_value;
pub(crate) mod not_a_number;
pub(crate) mod syntax;
pub(crate) mod undefined_variable;
pub(crate) mod unknown_function;

pub use error::Error;
pub use kind::Kind;
pub use position::Position;
