use crate::{error::Error, Value};
use rusty_v8 as v8;

pub trait AsLocal<T> {
    fn as_local(self, scope: &mut v8::HandleScope<'static, ()>) -> Result<T, Error>;
}

impl<'a> AsLocal<v8::Local<'a, v8::String>> for &str {
    fn as_local(
        self,
        scope: &mut v8::HandleScope<'static, ()>,
    ) -> Result<v8::Local<'a, v8::String>, Error> {
        let value = v8::String::new(scope, self).unwrap();

        return Ok(value);
    }
}

pub trait AsValue {
    fn as_value(&self, scope: &mut v8::HandleScope<'_>) -> Value;
}

impl AsValue for v8::Local<'_, v8::Value> {
    fn as_value(&self, scope: &mut v8::HandleScope<'_>) -> Value {
        Value::String(self.to_rust_string_lossy(scope))
    }
}
