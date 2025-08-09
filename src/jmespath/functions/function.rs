use super::{Parameter, ReturnValue};
use crate::{FunctionContext, Value};

/// A type that represents a JMESPath function.
pub trait Function {
    /// Returns the name of the function.
    fn get_name(&self) -> &str;
    /// Returns the function signature.
    fn get_signature(&self) -> &Vec<Parameter>;
    /// Invokes the function with the given arguments.
    fn execute(&self, args: &Vec<Value>, context: &dyn FunctionContext) -> ReturnValue;

    /// Returns the name of the given parameter to the function.
    ///
    /// Function authors are encouraged to implement this function
    /// for better error reporting. Otherwise, default names
    /// such as `param0`, `param1`, etc. are returned.
    ///
    fn get_parameter_name(&self, index: usize) -> String {
        format!("param{}", index)
    }
}

/// Utility helper to implement a JMESPath [`Function`] trait.
///
/// The function macro implements JMESPath functions as a struct
/// that implements the [`Function`] trait.
///
/// Please, scroll down below for more details.
///
/// # Example
///
/// ```rust
/// mod custom_functions {
///
///     use jmespath_community as jmespath;
///     use jmespath::function;
///
///     use jmespath::errors::Error as RuntimeError;
///
///     use jmespath::FunctionContext;
///     use jmespath::Value;
///
///     use jmespath::functions::ReturnValue;
///     use jmespath::functions::Function;
///
///     use jmespath::functions::DataType;
///     use jmespath::functions::ParamTypes::*;
///     use jmespath::functions::Parameter;
///     use jmespath::functions::Parameter::*;
///
///     function!(
///         add,
///         [
///             left => Required(Of(DataType::Number)),
///             right => Required(Of(DataType::Number))
///         ],
///         arguments,
///         {
///             // type checking has been performed by the runtime
///             // safe to unwrap
///
///             let i = arguments[0].as_f64().unwrap();
///             let j = arguments[1].as_f64().unwrap();
///
///             Value::from_f64(i + j)
///         }
///     );
/// }
/// ```
/// A [`Function`] is defined by:
/// - Its signature, specifying the type and arity of parameters.
/// - A code implementation.
///
/// A [`Function`] implementation can be defined in either of two ways:
///
/// - By supplying named arguments and a code block.
/// - By supplying a closure.
///
/// ## Specifying the function signature
///
/// The signature specifies the list and arity of each argument.
/// Although optional, it is recommended to also specify parameter names
/// for improved error reporting at runtime.
///
/// Use the [Parameter](crate::functions::Parameter) and [ParamTypes](crate::functions::ParamTypes) enums to succinctly
/// define parameter requirements.
///
/// Each parameter can be either [Parameter::Required](crate::functions::Parameter::Required),
/// [Parameter::Optional](crate::functions::Parameter::Optional) or [Parameter::Variadic](crate::functions::Parameter::Variadic).
///
/// An optional parameter MAY appear at any point in the function signature.
/// However, an optional parameter MUST be followed by zero or more other
/// optional parameters.
///
/// A variadic parameter MAY appear as the last parameter in the function signature.
/// It defines a parameter that can be repeated an arbitrary amount of times when the
/// function is invoked.
///
/// Additionaly, a parameter can be either [ParamTypes::Of](crate::functions::ParamTypes::Of) a specific [DataType](crate::functions::DataType),
/// or can be of [ParamTypes::Any](crate::functions::ParamTypes::Any) data type taken from a list.
///
/// ### Example
/// ```compile_fail
/// Required(Of(DataType::Array)),
/// Optional(Any(DataType::Number, DataType::String))
/// ...
/// ```
///
/// ## Supplying implementation with named arguments and code block
///
/// The code block will be invoked with at most two arguments:
///
/// The first one is a mandatory [`Vec<Value>`] vector of function arguments.
///
/// The second one is an optional [FunctionContext](crate::FunctionContext) implementation supplied
/// by the runtime. A function implementation can use the function context to
/// create a key function to dynamically invoke expression-types when necessary.
///
/// By specifying named arguments, you are instructing the macro how those two
/// parameters will be mapped at runtime to named arguments from your code block.
///
/// ### Example
///
/// The code sample above uses a named argument `arguments` and does not use the function context.
/// Here is a code sample that makes use of both arguments:
///
/// ```compile_fail
/// function!(my_custom_function, [ Required(Of(DataType::String)) ],
///   my_args, my_ctx, {
///     /// use my_args to access function arguments
///     /// use my_ctx as the function context
///     ...
///   });
/// ```
///
/// ## Supplying implementation as a closure
///
/// Specifying a closure is a simple way to implement a custom function.
/// The closure takes three parameters:
/// - A reference to the struct that implements this function.
/// - A [`Vec<crate::Value>`] vector of function arguments.
/// - A [`FunctionContext`] trait implementation.
///
/// Likewise, a function implementation can use the function context to
/// create a key function to dynamically invoke expression-types when necessary.
///
/// For instance, here is the code for the builtin `min_by` function:
///
/// ### Example
/// ```compile_fail
/// function!(min_by, [
///     elements => Required(Of(DataType::Array)),
///     expr => Required(Of(DataType::ExpRef))
///     ], |me: &min_by, args: &Vec<Value>, context: &dyn FunctionContext| {
///
///         let array = args[0].as_array().unwrap();
///         let ast = args[1].as_expref().unwrap();
///
///         let params = vec![Of(DataType::Number)];
///         let closure = context.create_by_function(&ast, &params, me, 1).unwrap();
///
///         let values = array
///             .iter()
///             .map(|x| closure(x))
///             .collect::<Result<Vec<Value>, RuntimeError>>()?;
///
///         match values.iter().min_by_key(|x| x.as_number()) {
///             None => Ok(Value::Null),
///             Some(num) => Ok(num.clone()),
///         }
///     }
/// );
/// ```
///
/// In the preceding example, the second argument to the [min_by](crate::functions::builtin::min_by) function is a [DataType::ExpRef](crate::functions::DataType::ExpRef) expression-type
/// that is used to create a "key" function using the [FunctionContext::create_by_function](crate::FunctionContext::create_by_function) function.
/// This returns a closure that the function implementation uses to filter the array that was
/// specified as the first argument to the function.
///
#[macro_export]
macro_rules! function {

    ($name:ident, [$($param:expr),*], $closure: expr) => {
        #[allow(non_camel_case_types)]
        pub struct $name {
            signature: Vec<Parameter>,
        }
        impl $name {
            pub fn new() -> Self {
                let signature = vec![ $($param),* ];
                // We convert a sequence of parameters to a string
                // by appending the lowercase initial of their categories:
                // - Required -> becomes "r"
                // - Optional -> becomes "o"
                // - Variadic -> becomes "v"
                //
                // We then match this against the following regex:
                // r"^r*(o+|v)?$"
                //
                let expression: &std::string::String = &signature
                    .iter()
                    .map(|p| match p {
                        Parameter::Required(_) => "r",
                        Parameter::Optional(_) => "o",
                        Parameter::Variadic(_) => "v",
                    })
                    .collect();

                let regex = regex::Regex::new(r"^r*(o+|v)?$").unwrap();
                let is_match: bool = regex.is_match(&expression);
                if !is_match {
                    panic!("The signature '{}' is invalid", &expression);
                }
                $name {
                    signature: signature,
                }
            }
        }
        impl Function for $name {
            fn get_name(&self) -> &str {
                stringify!($name)
            }
            fn get_signature(&self) -> &Vec<Parameter> {
                &self.signature
            }
            fn execute(&self, args: &Vec<Value>, context: &dyn FunctionContext) -> ReturnValue {
                $closure(self, args, context).map(|v| v.into())
            }
        }
    };

    ($name:ident, [$($param_name:ident=> $param:expr),*], $closure: expr) => {
        #[allow(non_camel_case_types)]
        pub struct $name {
            signature: Vec<Parameter>,
            parameter_names: Vec<std::string::String>,
        }
        impl $name {
            pub fn new() -> Self {
                let signature = vec![ $($param),* ];
                let parameter_names = vec![ $(stringify!($param_name).to_string()),* ];
                // We convert a sequence of parameters to a string
                // by appending the lowercase initial of their categories:
                // - Required -> becomes "r"
                // - Optional -> becomes "o"
                // - Variadic -> becomes "v"
                //
                // We then match this against the following regex:
                // r"^r*(o+|v)?$"
                //
                let expression: &std::string::String = &signature
                    .iter()
                    .map(|p| match p {
                        Parameter::Required(_) => "r",
                        Parameter::Optional(_) => "o",
                        Parameter::Variadic(_) => "v",
                    })
                    .collect();

                let regex = regex::Regex::new(r"^r*(o+|v)?$").unwrap();
                let is_match: bool = regex.is_match(&expression);
                if !is_match {
                    panic!("The signature '{}' is invalid", &expression);
                }
                $name {
                    signature,
                    parameter_names,
                }
            }
        }
        impl Function for $name {
            fn get_name(&self) -> &str {
                stringify!($name)
            }
            fn get_signature(&self) -> &Vec<Parameter> {
                &self.signature
            }
            fn execute(&self, args: &Vec<Value>, context: &dyn FunctionContext) -> ReturnValue {
                $closure(self, args, context).map(|v| v.into())
            }
            fn get_parameter_name(&self, index: usize) -> std::string::String {
                self.parameter_names[index].to_string()
            }
        }
    };

    ($name:ident, [$($param:expr),*], $args:ident $(, $ctx:ident )?, $body: block) => {
        #[allow(non_camel_case_types)]
        pub struct $name {
            signature: Vec<Parameter>,
        }
        impl $name {
            pub fn new() -> Self {
                let signature = vec![ $($param),* ];
                // We convert a sequence of parameters to a string
                // by appending the lowercase initial of their categories:
                // - Required -> becomes "r"
                // - Optional -> becomes "o"
                // - Variadic -> becomes "v"
                //
                // We then match this against the following regex:
                // r"^r*(o+|v)?$"
                //
                let expression: &std::string::String = &signature
                    .iter()
                    .map(|p| match p {
                        Parameter::Required(_) => "r",
                        Parameter::Optional(_) => "o",
                        Parameter::Variadic(_) => "v",
                    })
                    .collect();

                let regex = regex::Regex::new(r"^r*(o+|v)?$").unwrap();
                let is_match: bool = regex.is_match(&expression);
                if !is_match {
                    panic!("The signature '{}' is invalid", &expression);
                }
                $name {
                    signature: signature,
                }
            }
        }
        impl Function for $name {
            fn get_name(&self) -> &str {
                stringify!($name)
            }
            fn get_signature(&self) -> &Vec<Parameter> {
                &self.signature
            }
            fn execute(&self, args: &Vec<Value>, #[allow(unused_variables)] context: &dyn FunctionContext) -> ReturnValue {
                let $args = args;
                $( let $ctx = context; )?
                $body
            }
        }
    };

    ($name:ident, [$($param_name:ident=> $param:expr),*], $args:ident $(, $ctx:ident )?, $body: block) => {
        #[allow(non_camel_case_types)]
        pub struct $name {
            signature: Vec<Parameter>,
            parameter_names: Vec<std::string::String>,
        }
        impl $name {
            pub fn new() -> Self {
                let signature = vec![ $($param),* ];
                let parameter_names = vec![ $(stringify!($param_name).to_string()),* ];
                // We convert a sequence of parameters to a string
                // by appending the lowercase initial of their categories:
                // - Required -> becomes "r"
                // - Optional -> becomes "o"
                // - Variadic -> becomes "v"
                //
                // We then match this against the following regex:
                // r"^r*(o+|v)?$"
                //
                let expression: &std::string::String = &signature
                    .iter()
                    .map(|p| match p {
                        Parameter::Required(_) => "r",
                        Parameter::Optional(_) => "o",
                        Parameter::Variadic(_) => "v",
                    })
                    .collect();

                let regex = regex::Regex::new(r"^r*(o+|v)?$").unwrap();
                let is_match: bool = regex.is_match(&expression);
                if !is_match {
                    panic!("The signature '{}' is invalid", &expression);
                }
                $name {
                    signature,
                    parameter_names,
                }
            }
        }
        impl Function for $name {
            fn get_name(&self) -> &str {
                stringify!($name)
            }
            fn get_signature(&self) -> &Vec<Parameter> {
                &self.signature
            }
            fn execute(&self, args: &Vec<Value>, #[allow(unused_variables)] context: &dyn FunctionContext) -> ReturnValue {
                let $args = args;
                $( let $ctx = context; )?
                $body
            }
            fn get_parameter_name(&self, index: usize) -> std::string::String {
                self.parameter_names[index].to_string()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::function;
    use crate::AST;

    use crate::errors::Error as RuntimeError;

    use crate::FunctionContext;
    use crate::Value;

    use crate::functions::DataType;
    use crate::functions::Function;
    use crate::functions::ParamTypes::{self, *};
    use crate::functions::Parameter::{self, *};
    use crate::functions::ReturnValue;
    use crate::runtime::ByFunctionHolder;

    function!(
        no_params_required_args,
        [Required(Of(DataType::Number))],
        _args,
        { Ok(true.into()) }
    );
    function!(
        no_params_optional_args,
        [Optional(Of(DataType::Number))],
        _args,
        _ctx,
        { Ok(true.into()) }
    );
    function!(
        no_params_variadic_args,
        [Variadic(Of(DataType::Number))],
        _args,
        { Ok(true.into()) }
    );
    function!(
        no_params_required_closure,
        [Required(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );
    function!(
        no_params_optional_closure,
        [Optional(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );
    function!(
        no_params_variadic_closure,
        [Variadic(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );
    function!(
        params_required_args,
        [param => Required(Of(DataType::Number))],
        _args, _ctx, { Ok(true.into()) }
    );
    function!(
        params_optional_args,
        [param => Optional(Of(DataType::Number))],
        _args, _ctx, { Ok(true.into()) }
    );
    function!(
        params_variadic_args,
        [param => Variadic(Of(DataType::Number))],
        _args, { Ok(true.into()) }
    );
    function!(
        params_required_closure,
        [param => Required(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );
    function!(
        params_optional_closure,
        [param => Optional(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );
    function!(
        params_variadic_closure,
        [param => Variadic(Of(DataType::Number))],
        |_: &Self, _: &Vec<Value>, _: &dyn FunctionContext| { Ok(true) }
    );

    struct Fixture {
        pub args: Vec<Value>,
    }
    impl FunctionContext for Fixture {
        fn create_by_function(
            &'_ self,
            _: &AST,
            _: &Vec<ParamTypes>,
            _: &dyn Function,
            _: usize,
        ) -> Result<ByFunctionHolder<'_>, RuntimeError> {
            unimplemented!()
        }
    }

    fn setup() -> Fixture {
        Fixture {
            args: Value::map_into(vec![1, 2]),
        }
    }

    macro_rules! funcs {
        ($test:ident, $func:ident, $param:expr, $match:pat) => {
            #[test]
            fn $test() {
                let fixture = setup();
                let func = self::$func::new();
                assert_eq!(stringify!($func), func.get_name());
                assert_eq!($param, func.get_parameter_name(0));
                assert!(matches!(
                    func.execute(&fixture.args, &fixture),
                    Ok(Value::Boolean(true))
                ));
                assert!(matches!(func.get_signature()[..], $match));
            }
        };
    }

    funcs!(
        it_supports_custom_function_params_required_args,
        params_required_args,
        "param",
        [Required(Of(..))]
    );
    funcs!(
        it_supports_custom_function_params_optional_args,
        params_optional_args,
        "param",
        [Optional(Of(..))]
    );
    funcs!(
        it_supports_custom_function_params_variadic_args,
        params_variadic_args,
        "param",
        [Variadic(Of(..))]
    );
    funcs!(
        it_supports_custom_function_params_required_closure,
        params_required_closure,
        "param",
        [Required(Of(..))]
    );
    funcs!(
        it_supports_custom_function_params_optional_closure,
        params_optional_closure,
        "param",
        [Optional(Of(..))]
    );
    funcs!(
        it_supports_custom_function_params_variadic_closure,
        params_variadic_closure,
        "param",
        [Variadic(Of(..))]
    );

    funcs!(
        it_supports_custom_function_no_params_required_args,
        no_params_required_args,
        "param0",
        [Required(Of(..))]
    );
    funcs!(
        it_supports_custom_function_no_params_optional_args,
        no_params_optional_args,
        "param0",
        [Optional(Of(..))]
    );
    funcs!(
        it_supports_custom_function_no_params_variadic_args,
        no_params_variadic_args,
        "param0",
        [Variadic(Of(..))]
    );
    funcs!(
        it_supports_custom_function_no_params_required_closure,
        no_params_required_closure,
        "param0",
        [Required(Of(..))]
    );
    funcs!(
        it_supports_custom_function_no_params_optional_closure,
        no_params_optional_closure,
        "param0",
        [Optional(Of(..))]
    );
    funcs!(
        it_supports_custom_function_no_params_variadic_closure,
        no_params_variadic_closure,
        "param0",
        [Variadic(Of(..))]
    );
}
