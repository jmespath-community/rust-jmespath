pub mod abs;
pub mod avg;
pub mod ceil;
pub mod contains;
pub mod ends_with;
pub mod floor;
pub mod length;
pub mod min_by;
pub mod reverse;
pub mod starts_with;

#[cfg(test)]
mod test_utils {

    use crate::functions::{Function, RuntimeError};
    use crate::{FunctionContext, Runtime};

    pub(crate) struct Fixture {
        pub runtime: Runtime,
    }
    impl Fixture {
        pub(crate) fn setup() -> Self {
            let runtime = Runtime::create_runtime();
            Fixture { runtime }
        }
    }
    impl FunctionContext for Fixture {
        fn create_by_function<'a>(
            &'a self,
            _ast: &'a crate::AST,
            _params: &'a Vec<crate::functions::ParamTypes>,
            _function: &'a dyn Function,
            _param_index: usize,
        ) -> Result<crate::ByFunctionHolder<'a>, RuntimeError> {
            todo!()
        }
    }
}
