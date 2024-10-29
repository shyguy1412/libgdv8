mod environment;
mod value;

use environment::{unsafe_mut, V8Environment};
use godot::global::godot_print;
use rusty_v8 as v8;
use std::{
    ops::Deref,
    sync::{LazyLock, Mutex, MutexGuard, PoisonError},
};
pub use value::*;

static V8_INITIALIZED: Mutex<bool> = Mutex::new(false);

trait UnwrapEnvironment {
    fn unwrap_env(&mut self) -> &mut V8Environment<'static>;
}

impl UnwrapEnvironment
    for Result<
        MutexGuard<'_, Option<V8Environment<'static>>>,
        PoisonError<MutexGuard<'_, Option<V8Environment<'static>>>>,
    >
{
    fn unwrap_env(&mut self) -> &mut V8Environment<'static> {
        return self.as_mut().unwrap().as_mut().unwrap();
    }
}

pub struct Runtime {
    environment: LazyLock<Mutex<Option<V8Environment<'static>>>>,
}

impl Runtime {
    pub const fn prepare() -> Self {
        return Self {
            environment: LazyLock::new(|| Mutex::new(None)),
        };
    }
    pub fn new() -> Self {
        let mut runtime = Runtime::prepare();
        runtime.init();
        return runtime;
    }
    pub fn init(&mut self) {
        let mut v8_init = V8_INITIALIZED.lock().unwrap();
        if !*v8_init {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            *v8_init = true;
        }

        let mut env = self.environment.lock().unwrap();

        if env.is_none() {
            *env = Some(V8Environment::new());
        }
    }
    pub fn run_script(&mut self, source: &str) -> Option<v8::Local<'_, v8::Value>> {
        // Create a string containing the JavaScript source code.
        let env_lock = &mut self.environment.lock();
        let env = env_lock.unwrap_env();
        let scope = &mut v8::ContextScope::new(unsafe_mut(env.scope), *unsafe_mut(env.context));
        let result = v8::String::new(scope, source)
            .and_then(|code| v8::Script::compile(scope, code, None))
            .and_then(|script| script.run(scope));

        return result;
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        godot_print!("DROPPED RUNTIME");
        self.environment.lock().unwrap().as_mut().unwrap().drop();
        // self.run_script("'hello world'");
    }
}
