use super::types::DataType;

/// Represents the type for a parameter to a JMESPath [`super::Function`].
#[derive(Debug)]
pub enum ParamTypes {
    /// Represents a parameter of a given [`crate::functions::DataType`].
    /// # Example
    /// ```
    /// use jmespath::functions::DataType;
    /// use jmespath::functions::Parameter::*;
    /// use jmespath::functions::ParamTypes::*;
    /// let data_types = Optional(Of(DataType::String));
    /// ```
    ///
    /// See [`crate::functions::Parameter`].
    Of(DataType),
    /// Represents a parameter whose [`crate::functions::DataType`] is taken from a list.
    /// # Example
    /// ```
    /// use jmespath::functions::DataType;
    /// use jmespath::functions::Parameter::*;
    /// use jmespath::functions::ParamTypes::*;
    /// let data_types = Required(Any(vec![DataType::Array, DataType::Object, DataType::String]));
    /// ```
    ///
    /// See [`crate::functions::Parameter`].
    Any(Vec<DataType>),
}

/// Represents the definition for a parameter to a JMESPath [`super::Function`].
///
#[derive(Debug)]
pub enum Parameter {
    /// Represents a required parameter to a JMESPath [`super::Function`].
    ///
    /// # Example
    /// See [`crate::functions::ParamTypes`].
    Required(ParamTypes),
    /// Represents an optional parameter to a JMESPath [`super::Function`].
    ///
    /// An optional parameter MAY be present in the last parameters from
    /// a given function signature and MUST be followed by zero or more
    /// optional parameters.
    ///
    /// # Example
    /// See [`crate::functions::ParamTypes`].
    Optional(ParamTypes),
    /// Represents a variadic parameter to a JMESPath [`super::Function`].
    ///
    /// A variadic parameter MAY be present as the last parameter from
    /// a given function signature.
    ///
    /// # Example
    /// ```
    /// use jmespath::functions::DataType;
    /// use jmespath::functions::Parameter::*;
    /// use jmespath::functions::ParamTypes::*;
    /// let data_types = Variadic(Of(DataType::String));
    /// ```
    Variadic(ParamTypes),
}

impl Parameter {
    /// Returns `true` if the parameter is [`Optional`].
    ///
    /// [`Optional`]: Parameter::Optional
    #[must_use]
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(..))
    }
    /// Returns `true` if the parameter is [`Variadic`].
    ///
    /// [`Variadic`]: Parameter::Variadic
    #[must_use]
    pub fn is_variadic(&self) -> bool {
        matches!(self, Self::Variadic(..))
    }
    pub fn get_data_type(&self) -> Option<DataType> {
        let param_types = self.get_param_types();
        match param_types {
            ParamTypes::Of(t) => Some(*t),
            _ => None,
        }
    }
    pub fn get_data_types(&self) -> Option<&Vec<DataType>> {
        let param_types = self.get_param_types();
        match param_types {
            ParamTypes::Any(t) => Some(t),
            _ => None,
        }
    }
    pub fn get_param_types(&self) -> &ParamTypes {
        match self {
            Parameter::Required(t) => t,
            Parameter::Optional(t) => t,
            Parameter::Variadic(t) => t,
        }
    }
}

/// Helper functions for dealing with JMESPath [`super::Function`] signatures.
pub struct Signature {}
impl Signature {
    /// Returns true if the signature is variadic, _i.e_ if the
    /// last parameter may be repeated an arbitrary number of times.
    ///
    /// # Example
    ///
    /// ```
    /// use jmespath::functions::DataType::*;
    /// use jmespath::functions::Parameter::*;
    /// use jmespath::functions::ParamTypes::*;
    /// use jmespath::functions::Signature;
    ///
    /// let variadic = vec![Required(Of(Number)), Variadic(Of(String))];
    /// assert!(Signature::is_variadic(&variadic));
    /// ```
    pub fn is_variadic(parameters: &Vec<Parameter>) -> bool {
        match parameters.last() {
            Some(param) => param.is_variadic(),
            _ => false,
        }
    }
    /// Returns the minimum number of required parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use jmespath::functions::DataType::*;
    /// use jmespath::functions::Parameter::*;
    /// use jmespath::functions::ParamTypes::*;
    /// use jmespath::functions::Signature;
    ///
    /// let parameters = vec![Required(Of(Number))];
    /// assert_eq!(1, Signature::get_min_args_count(&parameters))
    /// ```
    pub fn get_min_args_count(parameters: &Vec<Parameter>) -> usize {
        parameters
            .iter()
            .filter(|x| {
                matches!(x, Parameter::Required(..)) || matches!(x, Parameter::Variadic(..))
            })
            .count()
    }
    /// Returns te maximum number of required and optional parameters, if any.
    ///
    pub fn get_max_args_count(parameters: &Vec<Parameter>) -> Option<usize> {
        if Self::is_variadic(parameters) {
            return None;
        }
        Some(parameters.len())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use super::DataType::*;
    use super::ParamTypes::*;
    use super::Parameter::*;

    #[test]
    fn is_variadic_false_on_empty_parameters_list() {
        let no_parameters = vec![];
        assert!(!Signature::is_variadic(&no_parameters));
    }
    #[test]
    fn is_variadic_false_on_required_or_optional_parameters_list() {
        let parameters = vec![Required(Of(Number)), Optional(Of(String))];
        assert!(!Signature::is_variadic(&parameters));
    }
    #[test]
    fn is_variadic() {
        let variadic = vec![Required(Of(Number)), Variadic(Of(String))];
        assert!(Signature::is_variadic(&variadic));
    }
    #[test]
    fn get_min_args_count() {
        let parameters = vec![Variadic(Of(Number))];
        assert_eq!(1, Signature::get_min_args_count(&parameters))
    }
    #[test]
    fn get_max_args_count() {
        let parameters = vec![Required(Of(Number)), Optional(Of(Number))];
        let max_count = Signature::get_max_args_count(&parameters);
        assert!(matches!(max_count, Some(2)));
    }
    #[test]
    fn get_max_args_count_is_none_for_variadic() {
        let parameters = vec![Variadic(Of(Number))];
        let max_count = Signature::get_max_args_count(&parameters);
        assert!(max_count.is_none());
    }
}
