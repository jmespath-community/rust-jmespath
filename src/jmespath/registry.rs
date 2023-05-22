use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::runtime::FunctionRegistrar;
use crate::JmesPathFunction;

lazy_static! {
    pub static ref REGISTRY: Box<Registry> = Box::new(Registry::create_registry());
}
pub struct Registry {
    functions: HashMap<String, Box<JmesPathFunction>>,
}
macro_rules! register {
    ($expr:expr, $ident:ident) => {
        $expr.register(Box::new(crate::functions::builtin::$ident::$ident::new()));
    };
}
impl Registry {
    fn new() -> Self {
        Registry {
            functions: HashMap::new(),
        }
    }
    pub(crate) fn create_registry() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_functions();
        registry
    }
    fn register_builtin_functions(&mut self) {
        register!(self, abs);
        register!(self, length);
        register!(self, min_by);
        register!(self, reverse);
    }
}
impl FunctionRegistrar for Registry {
    fn register(&mut self, func: Box<JmesPathFunction>) {
        let name = func.get_name().to_string();
        self.functions.insert(name, func);
    }
    fn get(&self, function_name: &str) -> Option<&Box<JmesPathFunction>> {
        self.functions.get(function_name)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::function;

    use crate::FunctionContext;
    use crate::Value;

    use crate::functions::Function;
    use crate::functions::ReturnValue;

    use crate::functions::DataType;
    use crate::functions::ParamTypes::*;
    use crate::functions::Parameter;
    use crate::functions::Parameter::*;

    function!(
        add,
        [
            left => Required(Of(DataType::Number)),
            right => Required(Of(DataType::Number))
        ],
        |_:&add, args: &Vec<Value>, _:&dyn FunctionContext|
        {
            // type checking has been performed by the runtime
            // safe to unwrap

            let i = args[0].as_f64().unwrap();
            let j = args[1].as_f64().unwrap();

            Value::from_f64(i+j)
        }
    );

    #[test]
    fn it_registers_function() {
        let add = Box::new(self::add::new());
        let mut registry = Registry::create_registry();
        registry.register(add);

        assert_eq!("add", registry.get("add").unwrap().get_name())
    }
}
