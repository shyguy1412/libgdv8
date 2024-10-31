mod environment;
mod error;
mod value;

use environment::V8Environment;
pub use error::*;
use godot::global::godot_print;
use rusty_v8 as v8;
use std::sync::Mutex;
pub use value::*;

static V8_INITIALIZED: Mutex<bool> = Mutex::new(false);

pub struct Runtime {
    environment: Option<V8Environment>,
}

impl Runtime {
    pub const fn prepare() -> Self {
        return Self { environment: None };
    }
    // pub fn new() -> Result<Self, Error> {
    //     let mut runtime = Runtime::prepare();
    //     runtime.init()?;
    //     return Ok(runtime);
    // }
    pub fn init(&mut self) -> Result<(), Error> {
        let mut v8_init = V8_INITIALIZED.lock().unwrap();
        if !*v8_init {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            *v8_init = true;
        }

        if self.environment.is_none() {
            self.environment = Some(V8Environment::new()?)
        }

        Ok(())
    }
    pub fn run_script(&self, source: &str) -> Result<v8::Local<'_, v8::Value>, Error> {
        let context_scope = match self.environment.as_ref() {
            Some(v) => v.context_scope(),
            None => return Err(Error::InvalidEnvironment),
        };

        if context_scope.is_none() {
            return Err(Error::InvalidEnvironment);
        };

        let context_scope = context_scope.unwrap();

        let result = v8::String::new(context_scope, source)
            .and_then(|code| v8::Script::compile(context_scope, code, None))
            .and_then(|script| script.run(context_scope));

        return match result {
            Some(v) => Ok(v.clone()),
            None => Err(Error::None),
        };
    }

    pub fn to_rust_string_lossy(&self, value: v8::Local<rusty_v8::Value>) -> Result<String, Error> {
        // Create a string containing the JavaScript source code.
        if self.environment.is_none() {
            return Err(Error::InvalidEnvironment);
        };

        let environment = self.environment.as_ref().unwrap();

        let context_scope = environment.context_scope();

        if context_scope.is_none() {
            return Err(Error::InvalidEnvironment);
        };

        let context_scope = context_scope.unwrap();

        Ok(value.to_rust_string_lossy(context_scope))
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.environment = None;
    }
}
