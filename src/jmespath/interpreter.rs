use crate::errors::error_builder::{
    ErrorBuilder, FunctionErrorBuilder, InvalidTypeErrorBuilder, NotANumberErrorBuilder,
    SyntaxErrorBuilder,
};
use crate::errors::invalid_type::InvalidTypeErrorBuilderFactory;
use crate::errors::not_a_number::NotANumberErrorBuilderFactory;
use crate::errors::syntax::SyntaxErrorBuilderFactory;
use crate::errors::{Error as RuntimeError, Kind, Position};
use crate::functions::{DataType, Function, ParamTypes, ReturnValue};
use crate::parser::{Slice, AST};
use crate::runtime::{ByFunctionHolder, Runtime};
use crate::scopes::Scopes;
use crate::value_option::ValueOption as _;
use crate::Map;
use crate::Value;
use crate::{FunctionContext, NodeType};

pub struct Interpreter<'a> {
    root: Option<&'a Value>,
    runtime: &'a Runtime,
    scopes: Option<Box<Scopes<'a>>>,
}
impl<'a> Interpreter<'a> {
    pub fn new(runtime: &'a Runtime, root: &'a Value) -> Self {
        Interpreter {
            root: Some(root),
            runtime,
            scopes: None,
        }
    }
    pub fn with_scope(&'a self, bindings: &'a Map<String, Value>) -> Self {
        let inner_scope = match &self.scopes {
            None => Box::new(Scopes::new(bindings)),
            Some(s) => Box::new(s.with_scope(bindings)),
        };
        Interpreter {
            root: self.root,
            runtime: self.runtime,
            scopes: Some(inner_scope),
        }
    }
    pub fn evaluate(&self, ast: &AST) -> ReturnValue {
        self.visit(ast, self.root.unwrap())
    }

    fn visit_raw_string(&self, ast: &AST, _: &Value) -> ReturnValue {
        Ok(ast.raw_string().into())
    }
    fn visit_current_node(&self, _: &AST, value: &Value) -> ReturnValue {
        Ok(value.clone())
    }
    fn visit_root_node(&self, _: &AST, _: &Value) -> ReturnValue {
        Ok(self.root.unwrap().clone())
    }

    fn visit_quoted_identifier(
        &self,
        ast: &AST,
        value: &Value,
        position: &Position,
    ) -> ReturnValue {
        let identifier = Self::unwrap_quoted_identifier(ast.identifier(), *position)?;
        Ok(match value {
            Value::Expression(_) => unreachable!(),
            Value::Object(v) => {
                if v.contains_key(&identifier) {
                    v[&identifier].clone()
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        })
    }

    fn visit_identifier(&self, ast: &AST, value: &Value) -> ReturnValue {
        let identifier = ast.identifier();
        Ok(match value {
            Value::Expression(_) => unreachable!(),
            Value::Object(v) => {
                if v.contains_key(identifier) {
                    v[identifier].clone()
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        })
    }

    fn visit_variable_ref(&self, ast: &AST, _: &Value) -> ReturnValue {
        let variable_name = ast.variable_ref();
        match self.scopes.as_ref().map(|x| x.get(variable_name)).flatten() {
            Some(v) => Ok(v.clone()),
            _ => Err(RuntimeError::undefined_variable(variable_name)),
        }
    }

    fn visit_expref(&self, nodes: &Vec<AST>, _: &Value) -> ReturnValue {
        assert_eq!(1, nodes.len());
        Ok(Value::Expression(nodes[0].clone()))
    }

    fn visit_multi_select_hash(&self, map: &Map<String, AST>, value: &Value) -> ReturnValue {
        let mut object: Map<String, Value> = Map::new();
        for item in map {
            // unwrap quoted identifier
            let key = if item.0.starts_with("\"") {
                Self::unwrap_quoted_identifier(&item.0, Position::default())?
            } else {
                item.0.clone()
            };
            let evaluated = self.visit(&item.1, value)?;
            object.insert(key, evaluated);
        }
        Ok(Value::Object(object))
    }

    fn visit_multi_select_list(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let mut array = Vec::new();
        for node in nodes {
            array.push(self.visit(node, value)?);
        }
        Ok(Value::Array(array))
    }

    fn visit_pipe_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        assert_eq!(2, nodes.len());
        let left = self.visit(&nodes[0], value)?;
        self.visit(&nodes[1], &left)
    }
    fn visit_sub_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        assert_eq!(2, nodes.len());
        let left = self.visit(&nodes[0], value)?;
        if let Value::Null = left {
            return Ok(Value::Null);
        }
        self.visit(&nodes[1], &left)
    }

    fn visit_hash_wildcard_projection(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let object = match &nodes[0].node_type {
            NodeType::None => Ok(value.clone()),
            _ => self.visit(&nodes[0], value),
        }?;
        Ok(match object {
            Value::Object(map) => {
                let values = map
                    .values()
                    .filter(|x| !x.is_null())
                    .map(|x| x.clone())
                    .collect::<Vec<Value>>();
                if let NodeType::None = nodes[1].node_type {
                    Value::Array(values)
                } else {
                    let mut result = Vec::new();
                    for item in values {
                        let right = self.visit(&nodes[1], &item)?;
                        if !right.is_null() {
                            result.push(right);
                        }
                    }
                    Value::Array(result)
                }
            }
            _ => Value::Null,
        })
    }
    fn visit_projection(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let left = match &nodes[0].node_type {
            NodeType::Filter(..) => self.filter(nodes, value),
            NodeType::Flatten => self.flatten(&nodes[1], value),
            NodeType::ListWildcard => self.list_wildcard(&nodes[1], value),
            NodeType::Slice(slice) => self.slice(slice, &nodes[1], value, nodes[0].position),
            _ => unreachable!(),
        }?;
        if let NodeType::None = nodes[2].node_type {
            return Ok(left);
        };
        let result = match left {
            Value::Array(array) => {
                let mut result = Vec::new();
                for item in array {
                    let right = self.visit(&nodes[2], &item)?;
                    if !right.is_null() {
                        result.push(right);
                    }
                }
                Value::Array(result)
            }
            Value::String(..) => {
                // a slice projection with a second argument
                // is really a sub-expression in disguise
                self.visit(&nodes[2], &left)?
            }
            _ => Value::Null,
        };
        Ok(result.into())
    }
    fn filter(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let array = match &nodes[1].node_type {
            NodeType::None => Ok(value.clone()),
            _ => self.visit(&nodes[1], value),
        }?;
        Ok(match array {
            Value::Array(array) => {
                let result = match &nodes[0].node_type {
                    NodeType::Filter(ast) => {
                        let mut result = Vec::new();
                        for item in array {
                            let filtered = self.visit(&ast[0], &item)?;
                            if filtered.is_truthy() {
                                result.push(item)
                            }
                        }
                        result
                    }
                    _ => unreachable!(),
                };
                Value::Array(result)
            }
            _ => Value::Null,
        })
    }
    fn flatten(&self, node: &AST, value: &Value) -> ReturnValue {
        // define a local 'flatten_array' function
        fn flatten_array<'b>(array: &Vec<Value>, result: &'b mut Vec<Value>) -> &'b Vec<Value> {
            for item in array {
                if let Value::Array(nested_array) = item {
                    for nested_item in nested_array {
                        result.push(nested_item.clone())
                    }
                } else {
                    result.push(item.clone())
                }
            }
            result
        }
        // flatten the current node
        let array = match &node.node_type {
            NodeType::None => Ok(value.clone()),
            _ => self.visit(node, value),
        }?;
        Ok(match array {
            Value::Array(array) => {
                let mut result: Vec<Value> = Vec::new();
                let flat = flatten_array(&array, &mut result);
                Value::Array(flat.clone())
            }
            _ => Value::Null,
        })
    }
    fn list_wildcard(&self, node: &AST, value: &Value) -> ReturnValue {
        let array: Result<Value, RuntimeError> = if matches!(node.node_type, NodeType::None) {
            Ok(value.clone())
        } else {
            self.visit(node, value)
        };
        Ok(array?.if_array().or_null())
    }
    fn slice(&self, slice: &Slice, node: &AST, value: &Value, position: Position) -> ReturnValue {
        // define a local function to compute slice parameters
        fn compute_slice_params(
            slice: &Slice,
            array_length: usize,
            position: Position,
        ) -> Result<(isize, isize, isize), RuntimeError> {
            let len = array_length as isize;
            let step = slice.step.unwrap_or(1);
            if step == 0 {
                return Err(RuntimeError::get_syntax_error_builder()
                    .at(position)
                    .set_kind(Kind::InvalidValue)
                    .for_reason("slice step cannot be 0")
                    .build());
            };
            let start = slice
                .start
                .map(|x| if x < 0 { x + len } else { x })
                .unwrap_or(if step > 0 {
                    0
                } else {
                    (array_length - 1) as isize
                });
            let stop = slice
                .stop
                .map(|x| if x < 0 { x + len } else { x })
                .unwrap_or(if step > 0 { array_length as isize } else { -1 });
            Ok((start, stop, step))
        }
        // define a local function to slice an array
        fn slice_array<T>(vector: &Vec<T>, start: isize, stop: isize, step: isize) -> Vec<T>
        where
            T: Clone,
        {
            let mut result: Vec<T> = Vec::new();
            let len = vector.len() as isize;
            let end = ((stop - start) as f64 / step as f64).ceil() as isize;
            for n in 0..end {
                let index = start + n * step;
                if index >= 0 && index < len {
                    result.push(vector[index as usize].clone());
                }
            }
            result
        }
        // slice the current array or string
        let array = match &node.node_type {
            NodeType::None => Ok(value.clone()),
            _ => self.visit(node, value),
        }?;
        match array {
            Value::Array(input) => {
                let params = compute_slice_params(slice, input.len(), position)?;
                let vector = slice_array(&input, params.0, params.1, params.2);
                Ok(Value::Array(vector))
            }
            Value::String(text) => {
                // slicing string by converting to an array of chars
                let characters: Vec<char> = text.chars().collect();
                let params = compute_slice_params(slice, characters.len(), position)?;
                let sliced = slice_array(&characters, params.0, params.1, params.2);
                Ok(Value::String(String::from_iter(sliced)))
            }
            _ => Ok(Value::Null),
        }
    }

    fn visit_index_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let array = match &nodes[0].node_type {
            NodeType::None => Ok(value.clone()),
            _ => self.visit(&nodes[0], value),
        }?;

        Ok(match array {
            Value::Array(v) => {
                let index = nodes[1].number();
                let index = if index >= 0 {
                    index
                } else {
                    index + v.len() as i32
                };
                match TryInto::<usize>::try_into(index) {
                    Ok(i) => {
                        if i < v.len() {
                            v[i].clone()
                        } else {
                            Value::Null
                        }
                    }
                    _ => Value::Null,
                }
            }
            _ => Value::Null,
        })
    }

    fn visit_json_value(&self, node: &str, position: &Position) -> ReturnValue {
        Value::from_json(node).map_err(|e| Self::map_err(e, *position))
    }

    fn visit_arithmetic_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        if let NodeType::None = nodes[0].node_type {
            return self.visit_arithmetic_unary(nodes, value);
        }
        self.visit_arithmetic_binary(nodes, value)
    }
    fn visit_arithmetic_unary(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        let right = self.visit(&nodes[2], value)?;
        let rhs = right.as_f64();

        if let None = rhs {
            let reason = format!("arithmetic expression required its right hand side to evaluate to a number, but received '{}' (of type '{}') instead", right, right.get_data_type());
            return Err(RuntimeError::get_syntax_error_builder()
                .at(nodes[1].position)
                .for_reason(&reason)
                .build());
        }
        let result = match &nodes[1].node_type {
            NodeType::Minus => Value::from_f64(-rhs.unwrap()),
            NodeType::Plus => Ok(right.clone()),

            _ => unreachable!(),
        };
        match result {
            Err(_) => {
                return Err(RuntimeError::get_not_a_number_error_builder()
                    .at(nodes[1].position)
                    .for_reason("the arithmetic expression evaluated to an invalid number")
                    .build());
            }
            ok => ok,
        }
    }
    fn visit_arithmetic_binary(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        assert_eq!(3, nodes.len());
        let left = self.visit(&nodes[0], value)?;
        let right = self.visit(&nodes[2], value)?;

        let lhs = left.as_f64();
        let rhs = right.as_f64();

        if let None = lhs {
            let reason = format!("arithmetic expression required its left hand side to evaluate to a number, but received '{}' (of type '{}') instead", left, left.get_data_type());
            return Err(RuntimeError::get_syntax_error_builder()
                .at(nodes[1].position)
                .for_reason(&reason)
                .build());
        }
        if let None = rhs {
            let reason = format!("arithmetic expression required its right hand side to evaluate to a number, but received '{}' (of type '{}') instead", right, right.get_data_type());
            return Err(RuntimeError::get_syntax_error_builder()
                .at(nodes[1].position)
                .for_reason(&reason)
                .build());
        }

        let result = match &nodes[1].node_type {
            NodeType::Divide => Value::from_f64(lhs.unwrap() / rhs.unwrap()),
            NodeType::Minus => Value::from_f64(lhs.unwrap() - rhs.unwrap()),
            NodeType::Multiply => Value::from_f64(lhs.unwrap() * rhs.unwrap()),
            NodeType::Plus => Value::from_f64(lhs.unwrap() + rhs.unwrap()),

            NodeType::Modulo => Value::from_f64(((lhs.unwrap() % rhs.unwrap()) as i64) as f64),
            NodeType::Div => Value::from_f64(((lhs.unwrap() / rhs.unwrap()) as i64) as f64),

            _ => unreachable!(),
        };
        match result {
            Err(_) => {
                return Err(RuntimeError::get_not_a_number_error_builder()
                    .at(nodes[1].position)
                    .for_reason("the arithmetic expression evaluated to an invalid number")
                    .build());
            }
            ok => ok,
        }
    }

    fn visit_comparator_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        assert_eq!(3, nodes.len());
        let left = self.visit(&nodes[0], value)?;
        let right = self.visit(&nodes[2], value)?;

        let requires_numbers = match &nodes[1].node_type {
            NodeType::Equal => false,
            NodeType::NotEqual => false,
            _ => true,
        };

        let lhs = left.as_f64();
        let rhs = right.as_f64();

        if let None = lhs {
            if requires_numbers {
                return Ok(Value::Null);
            }
        }
        if let None = rhs {
            if requires_numbers {
                return Ok(Value::Null);
            }
        }
        let compared: bool = match &nodes[1].node_type {
            NodeType::GreaterThan => lhs.unwrap() > rhs.unwrap(),
            NodeType::GreaterThanOrEqual => lhs.unwrap() >= rhs.unwrap(),
            NodeType::LessThanOrEqual => lhs.unwrap() <= rhs.unwrap(),
            NodeType::LessThan => lhs.unwrap() < rhs.unwrap(),
            NodeType::Equal => left == right,
            NodeType::NotEqual => left != right,
            _ => unreachable!(),
        };

        Ok(compared.into())
    }

    fn visit_logical_expression(&self, nodes: &Vec<AST>, value: &Value) -> ReturnValue {
        if nodes.len() == 2 {
            return self
                .visit(&nodes[1], value)
                .map(|x| Value::Boolean(!x.is_truthy()));
        }

        let left = self.visit(&nodes[0], value)?;
        let right = self.visit(&nodes[2], value)?;

        let compared: Value = match &nodes[1].node_type {
            NodeType::And => {
                if left.is_truthy() {
                    right
                } else {
                    left
                }
            }
            NodeType::Or => {
                if left.is_truthy() {
                    left
                } else {
                    right
                }
            }
            _ => unreachable!(),
        };

        Ok(compared.into())
    }

    fn visit_function_expression(&self, ast: &Vec<AST>, value: &Value) -> ReturnValue {
        let function_name = ast[0].function_name();
        let function_args = ast[1].function_arguments();

        let mut arguments = Vec::new();
        for function_arg in function_args {
            arguments.push(self.visit(function_arg, value)?);
        }
        self.runtime.call(function_name, &arguments, self)
    }

    fn visit_let_expression(&self, ast: &Vec<AST>, value: &Value) -> ReturnValue {
        let mut scopes: Map<String, Value> = Map::new();
        let bindings = ast[0].bindings();
        for index in 0..bindings.len() {
            if index % 2 != 0 {
                continue;
            }
            let name = bindings[index].variable_ref();
            let value = self.visit(&bindings[index + 1], value)?;
            scopes.insert(name.to_string(), value);
        }

        self.with_scope(&scopes).visit(&ast[1], value)
    }

    fn visit(&self, ast: &AST, value: &Value) -> ReturnValue {
        match &ast.node_type {
            NodeType::ArithmeticExpression(nodes) => self.visit_arithmetic_expression(nodes, value),
            NodeType::ComparatorExpression(nodes) => self.visit_comparator_expression(nodes, value),
            NodeType::CurrentNode => self.visit_current_node(ast, value),
            NodeType::Expression(nodes) => self.visit_expref(nodes, value),
            NodeType::FunctionExpression(nodes) => self.visit_function_expression(nodes, value),
            NodeType::HashWildcardProjection(nodes) => {
                self.visit_hash_wildcard_projection(nodes, value)
            }
            NodeType::IndexExpression(nodes) => self.visit_index_expression(nodes, value),
            NodeType::JsonValue(json) => self.visit_json_value(json, &ast.position),
            NodeType::LetExpression(nodes) => self.visit_let_expression(nodes, value),
            NodeType::LogicalExpression(nodes) => self.visit_logical_expression(nodes, value),
            NodeType::MultiSelectHash(nodes) => self.visit_multi_select_hash(nodes, value),
            NodeType::MultiSelectList(nodes) => self.visit_multi_select_list(nodes, value),
            NodeType::ParenExpression(nodes) => self.visit(&nodes[0], value),
            NodeType::PipeExpression(nodes) => self.visit_pipe_expression(nodes, value),
            NodeType::Projection(nodes) => self.visit_projection(nodes, value),
            NodeType::QuotedIdentifier(_name) => {
                self.visit_quoted_identifier(ast, value, &ast.position)
            }
            NodeType::RawString(_) => self.visit_raw_string(ast, value),
            NodeType::RootNode => self.visit_root_node(ast, value),
            NodeType::SubExpression(nodes) => self.visit_sub_expression(nodes, value),
            NodeType::UnquotedIdentifier(_name) => self.visit_identifier(ast, value),
            NodeType::VariableRef(_) => self.visit_variable_ref(ast, value),

            _ => {
                println!("{:?}", ast);
                unreachable!()
            }
        }
    }
    fn eval(&self, ast: &AST, root: &Value) -> ReturnValue {
        self.visit(ast, root)
    }
    fn unwrap_quoted_identifier(
        quoted_string: &String,
        position: Position,
    ) -> Result<String, RuntimeError> {
        // convert quoted identifier escape sequences
        let identifier = Value::from_json(quoted_string)
            .map_err(|err| Self::map_err(err, position))?
            .as_str()
            .unwrap()
            .to_string();
        Ok(identifier)
    }
    fn map_err(error: serde_json::Error, position: Position) -> RuntimeError {
        let message = format!("{}", error);
        RuntimeError::get_syntax_error_builder()
            .at(position)
            .for_reason(&message)
            .build()
    }
}
impl<'a> FunctionContext for Interpreter<'a> {
    fn create_by_function<'b>(
        &'b self,
        ast: &'b AST,
        param: &'b Vec<ParamTypes>,
        function: &'b dyn Function,
        param_index: usize,
    ) -> Result<ByFunctionHolder, RuntimeError> {
        let closure = move |value: &Value| -> Result<Value, RuntimeError> {
            let result = self.eval(ast, value);
            if result.is_err() {
                return result;
            }
            let value = result.unwrap();
            let data_types: Vec<DataType> = param
                .iter()
                .map(|x| match x {
                    ParamTypes::Of(t) => vec![*t],
                    ParamTypes::Of(t) => vec![*t],
                    ParamTypes::Any(v) => v.clone(),
                    ParamTypes::Any(v) => v.clone(),
                    _ => unreachable!(),
                })
                .flatten()
                .collect();
            if Runtime::matches_data_type(&value, &data_types) {
                Ok(value)
            } else {
                let err = RuntimeError::get_invalid_type_error_builder()
                    .for_function(function.get_name())
                    .for_expression_parameter(&function.get_parameter_name(param_index))
                    .expected_data_types(&data_types)
                    .received(&value)
                    .build();
                Err(err)
            }
        };
        Ok(ByFunctionHolder {
            closure: Box::new(closure),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{Kind, Position};
    use crate::functions::ReturnValue;
    use crate::parser::Slice;
    use crate::Value;
    use crate::{map, NodeType};

    use rstest::*;

    fn make_ast(node_type: NodeType) -> AST {
        AST::make(node_type, Position::new(0, 0))
    }

    fn setup(root: &Value, ast: &AST) -> ReturnValue {
        let runtime = Runtime::get_shared_runtime();
        let interpreter = Interpreter::new(&runtime, root);
        interpreter.evaluate(&ast)
    }
    fn from_json(text: &str) -> Value {
        Value::from_json(text).unwrap()
    }

    #[test]
    fn raw_string() {
        let ast = make_ast(NodeType::RawString("raw_string".to_string()));
        let result = setup(&from_json("{}"), &ast).unwrap();
        assert_eq!("raw_string", result.as_str().unwrap());
    }
    #[test]
    fn current_node() {
        let ast = make_ast(NodeType::CurrentNode);
        let result = setup(&from_json("{}"), &ast).unwrap();
        assert!(result.is_object());
    }
    #[test]
    fn root_node() {
        let ast = make_ast(NodeType::RootNode);
        let result = setup(&from_json("{}"), &ast).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn quoted_identifier() {
        let ast = make_ast(NodeType::QuotedIdentifier("\"foo bar\"".to_string()));
        let root = from_json(r#"{"foo bar": "foobar"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("\"foobar\"", result.to_json());
    }

    #[test]
    fn quoted_identifier_unicode() {
        let ast = make_ast(NodeType::QuotedIdentifier(
            r#""e\u0301le\u0301ment""#.to_string(),
        ));
        let root = from_json("{\"e\u{301}le\u{301}ment\": \"élément\"}");
        let result = setup(&root, &ast).unwrap();
        assert_eq!("élément", result.as_str().unwrap());
    }

    #[test]
    fn unquoted_identifier() {
        let ast = make_ast(NodeType::UnquotedIdentifier("foo".to_string()));
        let root = from_json(r#"{"foo": "bar"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("\"bar\"", result.to_json());
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn multi_select_hash() {
        let nodes = map![
            "foo".to_string() => make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            "bar".to_string() => make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::MultiSelectHash(nodes));

        let root = from_json(r#"{"foo": "foo", "bar": "bar", "baz": "baz"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("{\"foo\":\"foo\",\"bar\":\"bar\"}", result.to_json());
    }
    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn multi_select_hash() {
        let nodes = map![
            "foo".to_string() => make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            "bar".to_string() => make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::MultiSelectHash(nodes));

        let root = from_json(r#"{"foo": "foo", "bar": "bar", "baz": "baz"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("{\"bar\":\"bar\",\"foo\":\"foo\"}", result.to_json());
    }

    #[test]
    fn multi_select_list() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
            make_ast(NodeType::UnquotedIdentifier("baz".to_string())),
        ];
        let ast = make_ast(NodeType::MultiSelectList(nodes));

        let root = from_json(r#"{"foo": "foo", "bar": "bar", "baz": "baz"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("[\"foo\",\"bar\",\"baz\"]", result.to_json());
    }

    #[test]
    fn pipe_expression() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::PipeExpression(nodes));

        let root = from_json(r#"{"foo": {"bar": "baz"}}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("\"baz\"", result.to_json());
    }
    #[test]
    fn sub_expression() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::SubExpression(nodes));

        let root = from_json(r#"{"foo": {"bar": "baz"}}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("\"baz\"", result.to_json());
    }

    #[rstest]
    #[case(r#"[{"ok": true},{"ok": false}]"#, r#"[{"ok":true}]"#)]
    fn filter_projection(#[case] input: &str, #[case] expected: &str) {
        let nodes = vec![
            make_ast(NodeType::Filter(vec![make_ast(
                NodeType::UnquotedIdentifier("ok".to_string()),
            )])),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(input);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[rstest]
    #[case(r#"{"foo": "bar"}"#, "null")]
    #[case(
        r#"{"foo": [1, [2, [3], [4, 5], 6]]}"#,
        "[1.0,2.0,[3.0],[4.0,5.0],6.0]"
    )]
    fn flatten_projection(#[case] input: &str, #[case] expected: &str) {
        let nodes = vec![
            make_ast(NodeType::Flatten),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(input);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }
    #[test]
    fn flatten_projection_raw() {
        let nodes = vec![
            make_ast(NodeType::Flatten),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"[{"bar": "baz"}, [{"bar": "qux"}]]"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"[{"bar":"baz"},{"bar":"qux"}]"#, result.to_json());
    }
    #[test]
    fn flatten_projection_rhs() {
        let nodes = vec![
            make_ast(NodeType::Flatten),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"{"foo": [{"bar": "baz"}, [{"bar": "qux"}]]}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"["baz","qux"]"#, result.to_json());
    }

    #[rstest]
    #[case(vec![make_ast(NodeType::None), make_ast(NodeType::None)], r#"[[{"bar":{"bar":"baz"}},{"bar":{"bar":"qux"}}]]"#)]
    #[case(vec![make_ast(NodeType::UnquotedIdentifier("foo".to_string())), make_ast(NodeType::None)], r#"null"#)]
    #[case(vec![make_ast(NodeType::None), make_ast(NodeType::UnquotedIdentifier("bar".to_string()))], r#"[]"#)]
    #[case(vec![make_ast(NodeType::IndexExpression(vec![make_ast(NodeType::UnquotedIdentifier("foo".to_string())), make_ast(NodeType::Number(0))])), make_ast(NodeType::UnquotedIdentifier("bar".to_string()))], r#"["baz"]"#)]
    fn hash_wildcard_projection(#[case] nodes: Vec<AST>, #[case] expected: &str) {
        let ast = make_ast(NodeType::HashWildcardProjection(nodes));

        let root = from_json(r#"{"foo": [{"bar": {"bar": "baz"}}, {"bar": {"bar": "qux"}}]}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[rstest]
    #[case(r#"{"foo": "bar"}"#, "null")]
    #[case(r#"{"foo": [1, 2, 3]}"#, "[1.0,2.0,3.0]")]
    fn list_wildcard_projection(#[case] input: &str, #[case] expected: &str) {
        let nodes = vec![
            make_ast(NodeType::ListWildcard),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(input);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }
    #[test]
    fn list_wildcard_projection_raw() {
        let nodes = vec![
            make_ast(NodeType::ListWildcard),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"[{"bar": "baz"}, {"bar": "qux"}]"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"[{"bar":"baz"},{"bar":"qux"}]"#, result.to_json());
    }
    #[test]
    fn list_wildcard_projection_rhs() {
        let nodes = vec![
            make_ast(NodeType::ListWildcard),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"{"foo": [{"bar": "baz"}, {"bar": "qux"}]}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"["baz","qux"]"#, result.to_json());
    }

    #[test]
    fn slice_projection() {
        let nodes = vec![
            make_ast(NodeType::Slice(Slice {
                start: None,
                stop: None,
                step: Some(-2),
            })),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"[1, 2, 3]"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"[3.0,1.0]"#, result.to_json());
    }

    #[test]
    fn slice_projection_err() {
        let nodes = vec![
            make_ast(NodeType::Slice(Slice {
                start: None,
                stop: None,
                step: Some(0),
            })),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"[1, 2, 3]"#);
        let result = setup(&root, &ast).map_err(|e| e.kind);
        assert!(matches!(result, Err(Kind::InvalidValue)));
    }

    #[test]
    fn slice_string() {
        let nodes = vec![
            make_ast(NodeType::Slice(Slice {
                start: None,
                stop: None,
                step: Some(-1),
            })),
            make_ast(NodeType::None),
            make_ast(NodeType::None),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        // e       ́        l       e       ́        m       e       n      t
        // U+0065, U+0301, U+006C, U+0065, U+0301, U+006D, U+0065, U+06E, U+0074
        let root = from_json(r#""élément""#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#""tneḿeĺe""#, result.to_json());
    }

    #[rstest]
    #[case(Some(-5), None, None, r#"["baz","qux"]"#)]
    #[case(None, None, Some(-1), r#"["qux","baz"]"#)]
    fn slice_projection_rhs(
        #[case] start: Option<isize>,
        #[case] stop: Option<isize>,
        #[case] step: Option<isize>,
        #[case] expected: &str,
    ) {
        let nodes = vec![
            make_ast(NodeType::Slice(Slice { start, stop, step })),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::Projection(nodes));

        let root = from_json(r#"{"foo": [{"bar": "baz"}, {"bar": "qux"}]}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[rstest]
    #[case("true", "true")]
    fn json_value(#[case] expected: &str, #[case] input: &str) {
        let ast = make_ast(NodeType::JsonValue(input.to_string()));
        let root = map!("foo" => "bar").into();
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[test]
    fn json_value_err() {
        let ast = make_ast(NodeType::JsonValue(r#"{"foo":"ba"#.to_string()));
        let result = setup(&None.into(), &ast);
        assert!(result.is_err());
    }

    #[rstest]
    #[case(r#"{"bar":"baz"}"#, 0)]
    #[case(r#"{"bar":"qux"}"#, -1)]
    #[case("null", 100)]
    #[case("null", -100)]
    fn index_expression(#[case] expected: &str, #[case] input: i32) {
        let nodes = vec![make_ast(NodeType::None), make_ast(NodeType::Number(input))];
        let ast = make_ast(NodeType::IndexExpression(nodes));

        let root = from_json(r#"[{"bar": "baz"}, {"bar": "qux"}]"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[test]
    fn index_expression_not_an_array() {
        let nodes = vec![make_ast(NodeType::None), make_ast(NodeType::Number(0))];
        let ast = make_ast(NodeType::IndexExpression(nodes));

        let root = from_json(r#"{"bar": "baz"}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!("null", result.to_json());
    }

    #[test]
    fn index_expression_identifier() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::Number(-1)),
        ];
        let ast = make_ast(NodeType::IndexExpression(nodes));

        let root = from_json(r#"{"foo": [{"bar": "baz"}, {"bar": "qux"}]}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(r#"{"bar":"qux"}"#, result.to_json());
    }

    #[rstest]
    #[case(make_ast(NodeType::Plus), "23.0")]
    #[case(make_ast(NodeType::Minus), "19.0")]
    #[case(make_ast(NodeType::Multiply), "42.0")]
    #[case(make_ast(NodeType::Divide), "10.5")]
    #[case(make_ast(NodeType::Div), "10.0")]
    #[case(make_ast(NodeType::Modulo), "1.0")]
    fn arithmetic_expression(#[case] op: AST, #[case] expected: &str) {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            op,
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::ArithmeticExpression(nodes));

        let root = from_json(r#"{"foo": 21, "bar": 2}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[rstest]
    #[case(make_ast(NodeType::Plus), "42.0")]
    #[case(make_ast(NodeType::Minus), "-42.0")]
    fn arithmetic_unary(#[case] op: AST, #[case] expected: &str) {
        let nodes = vec![
            make_ast(NodeType::None),
            op,
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
        ];
        let ast = make_ast(NodeType::ArithmeticExpression(nodes));

        let root = from_json(r#"{"foo": 42}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.to_json());
    }

    #[rstest]
    #[case(make_ast(NodeType::Plus), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::Minus), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::Multiply), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::Divide), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::Plus), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::Minus), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::Multiply), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::Divide), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    fn arithmetic_expression_syntax(#[case] op: AST, #[case] left: Value, #[case] right: Value) {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            op,
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::ArithmeticExpression(nodes));

        let root = map!("foo" => left, "bar" => right).into();
        let result = setup(&root, &ast).map_err(|e| e.kind);
        assert!(matches!(result, Err(Kind::Syntax)));
    }

    #[test]
    fn arithmetic_expression_not_a_number() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            make_ast(NodeType::Divide),
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::ArithmeticExpression(nodes));

        let root = from_json(r#"{"foo": 21, "bar": 0}"#);
        let result = setup(&root, &ast).map_err(|e| e.kind);
        assert!(matches!(result, Err(Kind::NotANumber)));
    }

    #[rstest]
    #[case(make_ast(NodeType::Equal), false)]
    #[case(make_ast(NodeType::GreaterThan), true)]
    #[case(make_ast(NodeType::GreaterThanOrEqual), true)]
    #[case(make_ast(NodeType::LessThan), false)]
    #[case(make_ast(NodeType::LessThanOrEqual), false)]
    #[case(make_ast(NodeType::NotEqual), true)]
    fn comparator_expression(#[case] op: AST, #[case] expected: bool) {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            op,
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::ComparatorExpression(nodes));

        let root = from_json(r#"{"foo": 21, "bar": 2}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result.as_bool().unwrap());
    }

    #[rstest]
    #[case(make_ast(NodeType::LessThan), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::LessThanOrEqual), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::GreaterThan), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::GreaterThanOrEqual), Value::from_f64(23.0).unwrap(), "not_a_number".into())]
    #[case(make_ast(NodeType::LessThan), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::LessThanOrEqual), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::GreaterThan), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    #[case(make_ast(NodeType::GreaterThanOrEqual), "not_a_number".into(), Value::from_f64(23.0).unwrap())]
    fn comparator_expression_not_a_number(
        #[case] op: AST,
        #[case] left: Value,
        #[case] right: Value,
    ) {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            op,
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::ComparatorExpression(nodes));

        let root = map!("foo" => left, "bar" => right).into();
        let result = setup(&root, &ast).unwrap();
        assert_eq!(Value::Null, result);
    }

    #[rstest]
    #[case(make_ast(NodeType::And), Value::Null)]
    #[case(make_ast(NodeType::Or), Value::from_f64(21.0).unwrap())]
    fn logical_expression(#[case] op: AST, #[case] expected: Value) {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
            op,
            make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
        ];
        let ast = make_ast(NodeType::LogicalExpression(nodes));

        let root = from_json(r#"{"foo": 21, "bar": null}"#);
        let result = setup(&root, &ast).unwrap();
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(r#"{"foo": []}"#, true)]
    #[case(r#"{"foo": {}}"#, true)]
    #[case(r#"{"foo": ""}"#, true)]
    #[case(r#"{"foo": false}"#, true)]
    #[case(r#"{"foo": [1]}"#, false)]
    #[case(r#"{"foo": {"foo": "bar"}}"#, false)]
    #[case(r#"{"foo": "not empty"}"#, false)]
    #[case(r#"{"foo": true}"#, false)]
    fn logical_expression_unary(#[case] json: &str, #[case] expected: bool) {
        let nodes = vec![
            make_ast(NodeType::Not),
            make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
        ];
        let ast = make_ast(NodeType::LogicalExpression(nodes));

        let value = Value::from_json(json).unwrap();
        let result = setup(&value, &ast).unwrap();
        assert_eq!(expected, result.as_bool().unwrap());
    }

    #[test]
    fn function_expression() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("length".to_string())),
            make_ast(NodeType::FunctionArguments(vec![make_ast(
                NodeType::RawString("foo".to_string()),
            )])),
        ];
        let ast = make_ast(NodeType::FunctionExpression(nodes));

        let root = from_json(r#"{}"#);
        let result = setup(&root, &ast).unwrap();

        let expected = from_json("3");
        assert_eq!(expected, result);
    }
    #[test]
    fn function_expression_expref() {
        let nodes = vec![
            make_ast(NodeType::UnquotedIdentifier("min_by".to_string())),
            make_ast(NodeType::FunctionArguments(vec![
                make_ast(NodeType::UnquotedIdentifier("foo".to_string())),
                make_ast(NodeType::Expression(vec![make_ast(
                    NodeType::UnquotedIdentifier("age".to_string()),
                )])),
            ])),
        ];
        let ast = make_ast(NodeType::FunctionExpression(nodes));

        let root =
            from_json(r#"{"foo": [{"name": "alice", "age": 26}, {"name": "bob", "age": 31}]}"#);
        let result = setup(&root, &ast).unwrap();

        let expected = from_json(r#"{"name": "alice", "age": 26}"#);
        assert_eq!(expected, result);
    }

    #[test]
    fn let_expression() {
        let nodes = vec![
            make_ast(NodeType::LetBindings(vec![
                make_ast(NodeType::VariableRef("$foo".to_string())),
                make_ast(NodeType::UnquotedIdentifier("bar".to_string())),
                make_ast(NodeType::VariableRef("$baz".to_string())),
                make_ast(NodeType::UnquotedIdentifier("qux".to_string())),
            ])),
            make_ast(NodeType::VariableRef("$baz".to_string())),
        ];
        let ast = make_ast(NodeType::LetExpression(nodes));

        let root = from_json(r#"{"bar": "bar", "qux": "quux"}"#);
        let result = setup(&root, &ast).unwrap();

        let expected = from_json(r#""quux""#);
        assert_eq!(expected, result);
    }

    #[test]
    fn let_expression_undefined_variable() {
        let ast = make_ast(NodeType::VariableRef("$baz".to_string()));

        let root = from_json(r#"{"bar": "bar", "qux": "quux"}"#);
        let result = setup(&root, &ast).map_err(|x| x.kind);

        assert!(result.is_err());
        match result {
            Err(err) => assert_eq!(Kind::UndefinedVariable, err),
            _ => assert!(false),
        }
    }
}
