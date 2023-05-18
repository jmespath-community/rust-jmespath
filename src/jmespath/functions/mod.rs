mod function;
mod types;

pub(crate) mod signature;

/// This module implements the builtin JMESPath functions.
pub mod builtin;

pub use crate::errors::Error as RuntimeError;

use crate::Value;
pub type ReturnValue = Result<Value, RuntimeError>;

pub use function::Function;
pub use signature::ParamTypes;
pub use signature::Parameter;
pub use signature::Signature;
pub use types::DataType;
