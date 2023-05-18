use crate::Map;
use crate::Value;

#[derive(Debug)]
pub(crate) struct Scopes<'a> {
    bindings: &'a Map<String, Value>,
    scopes: Option<&'a Self>,
}
impl<'a> Scopes<'a> {
    pub fn new(bindings: &'a Map<String, Value>) -> Self {
        Scopes {
            bindings,
            scopes: None,
        }
    }
    pub fn with_scope(&'a self, bindings: &'a Map<String, Value>) -> Self {
        Scopes {
            bindings: bindings,
            scopes: Some(self),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&Value> {
        match self.bindings.get(identifier) {
            Some(x) => Some(x),
            None => match self.scopes {
                Some(scopes) => scopes.get(identifier),
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::map;

    use super::*;

    #[test]
    fn scopes() {
        let binding = map!("foo".to_string() => "bar".into());
        let scopes = Scopes::new(&binding);

        match scopes.get("foo") {
            Some(Value::String(s)) => assert_eq!("bar", s),
            _ => assert!(false),
        }

        assert!(matches!(scopes.get("none"), None));
    }

    #[test]
    fn with_scope() {
        let inner_bindings = map!("foo".to_string() => "bar".into());
        let inner_scopes = Scopes::new(&inner_bindings);

        let binding = map!("bar".to_string() => "baz".into());
        let scopes = inner_scopes.with_scope(&binding);

        match scopes.get("bar") {
            Some(Value::String(s)) => assert_eq!("baz", s),
            _ => assert!(false),
        }
        match scopes.get("foo") {
            Some(Value::String(s)) => assert_eq!("bar", s),
            _ => assert!(false),
        }
    }
}
