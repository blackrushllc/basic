use basil_common::{BasilError, Result};
use basil_bytecode::{ObjectRef, Value};

// Minimal in-crate stub to remove external basil-objects dependency.
// Objects are not supported in the lean Basic build.
#[derive(Default)]
pub struct Registry;

impl Registry {
    pub fn new() -> Self { Registry }
    pub fn make(&self, _type_name: &str, _args: &[Value]) -> Result<ObjectRef> {
        Err(BasilError("Object system is not available in this build".into()))
    }
}

pub fn register_objects(_reg: &mut Registry) {
    // no-op in lean build
}
