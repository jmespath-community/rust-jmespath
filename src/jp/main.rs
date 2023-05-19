use std::{
    env,
    io::{self, Read},
};

use jmespath_community as jmespath;
use jmespath::{search, Value};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut expression = &"outer.foo || outer.bar".to_string();
    if args.len() > 1 {
        expression = &args[1];
    }
    println!("expression: {}", expression);

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read from standard input");

    // Remove extraneous newlines
    let buffer = buffer.trim_end_matches(|c| c == '\r' || c == '\n');
    println!("input: {:?}", buffer);

    let value = Value::from_json(&buffer).unwrap();
    match jmespath::parse(expression) {
        Ok(ast) => println!("{}", ast),
        Err(err) => println!("{}", err),
    }
    let result = search(expression, &value);
    match result {
        Ok(v) => println!("{}", v.to_json()),
        Err(err) => println!("{}", err),
    }
}
