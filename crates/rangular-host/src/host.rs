use crate::error::HostError;
use crate::value::Value;

pub trait Host {
    fn get(&self, name: &str) -> Option<Value>;

    /// # Errors
    ///
    /// When the property is read-only or the value type is rejected.
    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        let _ = (name, value);
        Ok(())
    }

    /// # Errors
    ///
    /// When the named handler rejects the call.
    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError>;
}
