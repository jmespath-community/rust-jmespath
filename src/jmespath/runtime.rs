use crate::errors::error_builder::{ErrorBuilder, FunctionErrorBuilder, InvalidTypeErrorBuilder};
use crate::errors::invalid_type::InvalidTypeErrorBuilderFactory;
use crate::errors::Error as RuntimeError;
use crate::functions::{DataType, Function, ParamTypes, Parameter, Signature};
use crate::interpreter::Interpreter;
use crate::registry::{Registry, REGISTRY};
use crate::{functions::ReturnValue, Value, AST};
use crate::{parse, JmesPathFunction};

/// Represents an expression type as runtime.
///
/// This supports the "_by" functions by holding a closure.
/// A custom function that supports expression-types can call
/// the [`crate::FunctionContext::create_by_function()`] function
/// to create an instance of the [`ByFunctionHolder`] type for
/// subsequent use.
///
/// # Example
///
/// See [`crate::function`] for more details.
pub struct ByFunctionHolder<'a> {
    /// A "key" function on a [`Value`].
    pub(crate) closure: Box<dyn Fn(&Value) -> ReturnValue + 'a>,
}
impl<'a> ByFunctionHolder<'a> {
    /// Invokes the "key" function on a [`Value`].
    pub fn call(&self, value: &Value) -> ReturnValue {
        (self.closure)(value)
    }
}
impl<'a> std::ops::Deref for ByFunctionHolder<'a> {
    type Target = dyn Fn(&Value) -> ReturnValue + 'a;

    fn deref(&self) -> &Self::Target {
        &*self.closure
    }
}
/// A type that represents a context accessible to JMESPath function implementations.
pub trait FunctionContext {
    /// Creates a closure to supports the "_by" functions that operate on expression types.
    ///
    /// An instance of the [`FunctionContext`] trait is supplied at runtime to the JMESPath
    /// function implementation. The function implementation then uses this function to
    /// create a closure that can be used to dynamically invoke expression-types.
    ///
    /// # Parameters
    ///
    /// * `ast` - an instance of the [`AST`](crate::parser::AST) abstract syntax tree node
    /// associated with an expression-type.
    ///
    /// * `params` - a [`Vec<crate::functions::ParamTypes>`] vector of parameters.
    /// The runtime will ensure that evaluating the key function will result in
    /// valid expected datatypes.
    ///
    /// * `function` -  a reference to the [`Function`] calling this method. The
    /// runtime calls the [`crate::functions::Function::get_name()`] function
    /// to retrieve the function name for the purpose of error reporting.
    ///
    /// * `parameter_index` - a zero-based index of the function parameter associated
    /// with the expression-type argument to the JMESPath function. The runtime calls
    /// the [`crate::functions::Function::get_parameter_name()`] function implementation
    /// to retrieve the correspdonding parameter name for the purpose of error reporting.
    ///
    /// # Example
    /// See documentation for [`crate::function`] macro.
    fn create_by_function<'a>(
        &'a self,
        ast: &'a AST,
        params: &'a Vec<ParamTypes>,
        function: &'a dyn Function,
        param_index: usize,
    ) -> Result<ByFunctionHolder<'a>, RuntimeError>;
}

/// A type that represents a registry of JMESPath functions.
pub trait FunctionRegistrar {
    /// Registers custom functions to make them available to JMESPath expressions.
    fn register(&mut self, function: Box<JmesPathFunction>);
    /// Retrieves a reference to a registered JMESPath function.
    fn get(&self, function_name: &str) -> Option<&Box<JmesPathFunction>>;
}

/// Represents a processing runtime for JMESPath function evaluation.
pub struct Runtime {
    shared_registry: &'static Box<Registry>,
    registry: Option<Box<Registry>>,
}
impl Runtime {
    /// Returns a static shared [`Runtime`] with all builtin [`Function`]
    /// types registered.
    pub fn get_shared_runtime() -> Self {
        Runtime {
            shared_registry: &REGISTRY,
            registry: None,
        }
    }
    /// Create a new instance of the [`Runtime`] type with all
    /// builtin [`Function`] types registered.
    ///
    /// Use the [register](crate::FunctionRegistrar::register()) function
    /// to register new custom functions.
    pub fn create_runtime() -> Self {
        Runtime {
            shared_registry: &REGISTRY,
            registry: Some(Box::new(Registry::create_registry())),
        }
    }
    /// Parses and evaluate a JMESPath expression.
    pub fn search(&self, expression: &str, root: &Value) -> ReturnValue {
        let ast = parse(expression)?;
        self.search_ast(&ast, root)
    }
    /// Evaluates a parsed JMESPath expression.
    pub fn search_ast(&self, ast: &AST, root: &Value) -> ReturnValue {
        let interpreter = Interpreter::new(self, root);
        interpreter.evaluate(&ast)
    }
    pub(crate) fn call(
        &self,
        fname: &str,
        args: &Vec<Value>,
        context: &dyn FunctionContext,
    ) -> ReturnValue {
        if let Some(func) = self.get(fname) {
            Self::ensure_arity(func, args)?;
            Self::ensure_type(func, args)?;
            return func.execute(args, context);
        }
        // unknown function
        Err(RuntimeError::unknown_function(fname))
    }
    fn ensure_arity(func: &Box<JmesPathFunction>, args: &Vec<Value>) -> Result<(), RuntimeError> {
        let params = func.get_signature();
        let function_name = func.get_name();

        let count = args.len();
        let is_variadic = Signature::is_variadic(params);
        let max_count = Signature::get_max_args_count(params);
        let min_count = Signature::get_min_args_count(params);

        if count < min_count {
            return Err(RuntimeError::too_few_arguments(
                function_name,
                min_count,
                count,
                is_variadic,
            ));
        }

        if let Some(n) = max_count {
            if count > n {
                return Err(RuntimeError::too_many_arguments(function_name, n, count));
            }
        }

        Ok(())
    }
    pub fn ensure_type(
        func: &Box<JmesPathFunction>,
        args: &Vec<Value>,
    ) -> Result<(), RuntimeError> {
        let params = func.get_signature();
        let function_name = func.get_name();

        let mut last_index: usize = 0;

        // handle specified parameters

        for i in 0..args.len() {
            if i >= params.len() {
                break;
            }

            last_index = i;

            Self::ensure_matches_parameter(
                function_name,
                &func.get_parameter_name(i),
                &args[i],
                &params[i],
            )?;
        }

        // handle additional variadic parameters

        let param = &params.last();

        while {
            last_index += 1;
            last_index < args.len() - 1
        } {
            assert!(param.is_some());
            assert!(matches!(param.unwrap(), Parameter::Variadic(..)));
            Self::ensure_matches_parameter(
                function_name,
                &func.get_parameter_name(params.len() - 1),
                &args[last_index],
                param.unwrap(),
            )?;
            last_index += 1;
        }

        Ok(())
    }
    pub fn ensure_matches_parameter(
        function_name: &str,
        parameter_name: &str,
        arg: &Value,
        param: &Parameter,
    ) -> Result<(), RuntimeError> {
        match param.get_param_types() {
            ParamTypes::Of(t) => {
                let v = vec![*t];
                Self::ensure_matches_data_type(function_name, parameter_name, arg, &v)
            }
            ParamTypes::Any(v) => {
                Self::ensure_matches_data_type(function_name, parameter_name, arg, v)
            }
        }
    }
    pub fn ensure_matches_data_type(
        function_name: &str,
        parameter_name: &str,
        arg: &Value,
        data_types: &Vec<DataType>,
    ) -> Result<(), RuntimeError> {
        if Self::matches_data_type(arg, data_types) {
            return Ok(());
        }

        let err = RuntimeError::get_invalid_type_error_builder()
            .for_function(function_name)
            .for_parameter(parameter_name)
            .expected_data_types(data_types)
            .received(arg)
            .build();

        Err(err)
    }
    pub(crate) fn matches_data_type(arg: &Value, data_types: &Vec<DataType>) -> bool {
        data_types
            .iter()
            .map(|x| match x {
                DataType::Any => true,
                DataType::Null => false,

                DataType::Array => arg.is_array(),
                DataType::Boolean => arg.is_bool(),
                DataType::ExpRef => arg.is_expression(),
                DataType::Number => arg.is_number(),
                DataType::Object => arg.is_object(),
                DataType::String => arg.is_str(),
            })
            .any(|x| x)
    }
}
impl FunctionRegistrar for Runtime {
    fn register(&mut self, function: Box<JmesPathFunction>) {
        if let Some(cell) = &mut self.registry {
            let registry = cell.as_mut();
            registry.register(function);
        } else {
            panic!("Cannot update an immutable shared registry!");
        }
    }

    fn get(&self, function_name: &str) -> Option<&Box<JmesPathFunction>> {
        match &self.registry {
            Some(cell) => cell.get(function_name),
            None => self.shared_registry.get(function_name),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        errors::{
            Kind::{self, *},
            Position,
        },
        NodeType,
    };

    use super::*;

    mod test_functions {

        use crate::function;

        use crate::FunctionContext;
        use crate::Value;

        use crate::functions::DataType::{self, *};
        use crate::functions::Function;
        use crate::functions::ParamTypes::{self, *};
        use crate::functions::Parameter::{self, *};
        use crate::functions::ReturnValue;

        function!(
            add,
            [
                lhs => Required(Of(Number)),
                rhs => Required(Of(Number))
            ],
            arguments,
            {
                // type checking has been performed by the runtime
                // safe to unwrap

                let i = arguments[0].as_f64().unwrap();
                let j = arguments[1].as_f64().unwrap();

                Value::from_f64(i + j)
            }
        );

        function!(sum, [
            args => Variadic(Of(Number))
            ], arguments, {
            // type checking has been performed by the runtime
            // safe to unwrap

            let sum = arguments
                .iter()
                .fold(0.0, |acc, cur| acc + cur.as_f64().unwrap());

            Value::from_f64(sum)
        });

        function!(
            by,
            [
                expr => Required(Of(DataType::ExpRef)),
                array => Optional(ParamTypes::Any(vec![
                    DataType::Array,
                    DataType::Boolean,
                    DataType::Number,
                    DataType::Object,
                    DataType::String,
                ]))
            ],
            |me: &by, args: &Vec<Value>, context: &dyn FunctionContext| {
                let ast = args[0].as_expref().unwrap();
                let params = vec![ParamTypes::Of(DataType::String)];
                let closure = context.create_by_function(&ast, &params, me, 1)?;
                closure.call(&args[1])
            }
        );
    }

    struct Fixture {
        pub runtime: Runtime,
    }
    impl FunctionContext for Fixture {
        fn create_by_function(
            &self,
            _: &AST,
            _: &Vec<ParamTypes>,
            _: &dyn Function,
            _: usize,
        ) -> Result<ByFunctionHolder, RuntimeError> {
            let closure = |_: &Value| Ok(Value::String("by_result".to_string()));
            Ok(ByFunctionHolder {
                closure: Box::new(closure),
            })
        }
    }

    fn setup() -> Fixture {
        let add_function: Box<JmesPathFunction> = Box::new(test_functions::add::new());
        let by_function: Box<JmesPathFunction> = Box::new(test_functions::by::new());
        let sum_function: Box<JmesPathFunction> = Box::new(test_functions::sum::new());

        let mut runtime = Runtime::create_runtime();
        runtime.register(add_function);
        runtime.register(by_function);
        runtime.register(sum_function);
        Fixture { runtime }
    }

    #[test]
    fn register_and_call_custom_add_function() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = 2.into();
        let arg2 = 2.into();

        let fname = "add";
        let args = vec![arg1, arg2];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert!(result.is_number());
        assert_eq!(4.0, result.as_f64().unwrap());
    }

    #[test]
    fn register_and_call_custom_by_function() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = Value::Expression(AST::make(
            NodeType::RawString("foo".to_string()),
            Position::new(1, 1),
        ));
        let arg2 = 2.into();

        let fname = "by";
        let args: Vec<Value> = vec![arg1, arg2];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert_eq!("by_result", result);
    }

    #[test]
    fn register_and_call_custom_sum_function() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = 1.into();
        let arg2 = 2i8.into();
        let arg3 = 3i16.into();

        let fname = "sum";
        let args = vec![arg1, arg2, arg3];
        let result = fixture.runtime.call(fname, &args, context).unwrap();

        assert!(result.is_number());
        assert_eq!(6.0, result.as_f64().unwrap());
    }

    #[test]
    fn unknown_function() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;
        let result = fixture
            .runtime
            .call("unknown", &Vec::new(), context)
            .map_err(|e| e.kind);

        assert!(matches!(result, Err(UnknownFunction)));
    }

    #[test]
    fn invalid_arity_too_few_arguments() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = 2.into();

        let fname = "add";
        let not_enough_args = vec![arg1];
        let result = fixture
            .runtime
            .call(fname, &not_enough_args, context)
            .map_err(|e| e.kind);

        assert!(matches!(result, Err(InvalidArity)));
    }

    #[test]
    fn invalid_arity_too_many_arguments() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = 2.into();
        let arg2 = 4.into();
        let arg3 = 6.into();

        let fname = "add";
        let too_many_args = vec![arg1, arg2, arg3];
        let result = fixture
            .runtime
            .call(fname, &too_many_args, context)
            .map_err(|x| x.kind);

        assert!(matches!(result, Err(InvalidArity)));
    }

    #[test]
    fn invalid_type() {
        let fixture = setup();
        let context: &dyn FunctionContext = &fixture;

        // call function

        let arg1 = "two point zero".into();
        let arg2 = "three point zero".into();

        let fname = "add";
        let args = vec![arg1, arg2];

        let dt = vec![DataType::Number];
        let res = Runtime::ensure_matches_data_type(fname, "$width", &args[0], &dt);
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap().kind, Kind::InvalidType));

        let result = fixture
            .runtime
            .call(fname, &args, context)
            .map_err(|x| x.kind);

        assert!(matches!(result, Err(InvalidType)));
    }
}
