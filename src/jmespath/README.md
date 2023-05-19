JMESPath Community is [the community implementation](https://jmespath.site/) of JMESPath, a query and transformation language for JSON.

# Evaluating JMESPath Expression
Use the [search](https://docs.rs/jmespath_community/0.1.0/jmespath_community/fn.search.html) function to evaluate a JMESPath expression.

## Example
```rust
use jmespath_community as jmespath;
use jmespath::{search, Value};

/ Parse some JSON data into a JMESPath variable
et json_str = "{\"foo\":{\"bar\":{\"baz\":true}}}";
et data = Value::from_json(json_str).unwrap();

et result = search("foo.bar | baz", &data).unwrap();
ssert_eq!(true, result);
```

A JMESPath expression can be parsed once and evaluated
multiple times using the [parse](https://docs.rs/jmespath_community/0.1.0/jmespath_community/fn.parse.html) function.

## Example
```rust
use jmespath_community as jmespath;
use jmespath::{parse, Value};

let ast = parse("foo").unwrap();
let data = Value::from_json(r#"{"foo": "bar"}"#).unwrap();
let result = ast.search(&data).unwrap();
assert_eq!("bar", result);
```

# Registering Custom Functions

JMESPath Community comes with a host of useful [builtin functions](https://jmespath.site/main/#functions).
However, it can be extended with [third-party functions](https://docs.rs/jmespath_community/0.1.0/jmespath_community/macro.function.html).

## Example
```rust
mod custom_functions {

	use jmespath_community as jmespath;

    use jmespath::function;

    use jmespath::errors::Error as RuntimeError;

    use jmespath::FunctionContext;
    use jmespath::Value;

    use jmespath::functions::ReturnValue;
    use jmespath::functions::Function;

    use jmespath::functions::DataType;
    use jmespath::functions::ParamTypes::*;
    use jmespath::functions::Parameter;
    use jmespath::functions::Parameter::*;

    function!(
        add,
        [
            left => Required(Of(DataType::Number)),
            right => Required(Of(DataType::Number))
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
}
```

Create a new instance of the JMESPath [Runtime](https://docs.rs/jmespath_community/0.1.0/jmespath_community/struct.Runtime.html) object and
register your custom function:

## Example
```compile_fail
use jmespath_community as jmespath;
use jmespath::FunctionRegistrar;
use jmespath::{Runtime, Value};

let add = Box::new(custom_functions::add::new());
let mut runtime = Runtime::create_runtime();
runtime.register(add);

let expression = "foo";
let root = Value::Null;
let result = runtime.search(expression, &root).unwrap();

assert_eq!(None, result);
```
