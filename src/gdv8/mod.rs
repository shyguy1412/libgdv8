mod environment;
mod error;
mod value;

use environment::{AsLocal, V8Environment};
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
        let env = match &self.environment {
            Some(v) => v,
            None => return Err(Error::InvalidEnvironment),
        };

        let scope = env.context_scope()?;

        let scope = &mut v8::TryCatch::new(scope);

        let source = source.as_local(scope)?;
        let name = "name".as_local(scope)?.into();

        let result = v8::Script::compile(scope, source, None).and_then(|script| script.run(scope));

        if scope.has_caught() {
            let message = scope.exception().unwrap().to_rust_string_lossy(scope);
            let classname = scope
                .exception()
                .unwrap()
                .to_object(scope)
                .unwrap()
                .get(scope, name)
                .unwrap()
                .to_rust_string_lossy(scope);
            godot_print!("{}, {}", message, classname);
        }

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

        let context_scope = environment.context_scope()?;

        Ok(value.to_rust_string_lossy(context_scope))
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.environment = None;
    }
}
