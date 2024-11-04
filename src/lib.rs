mod context;
mod error;
mod helper;
mod value;

use std::{cell::OnceCell, collections::HashMap, sync::Mutex};
use rusty_v8 as v8;

pub use context::Context;
pub use error::Error;
pub use value::*;

static V8_RUNTIME: Mutex<OnceCell<Runtime>> = Mutex::new(OnceCell::new());

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

struct Runtime {
    isolate_ptr: *mut v8::OwnedIsolate,
    registry: HashMap<u64, HashMap<String, Callable>>,
}

pub enum Callable {
    Godot(godot::builtin::Callable),
    Closure(Box<dyn Fn(Vec<Value>) -> Value>),
}

impl Runtime {
    pub fn new() -> Self {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        Self {
            isolate_ptr: Box::leak(Box::new(v8::Isolate::new(v8::CreateParams::default()))),
            registry: HashMap::new(),
        }
    }
    pub fn get_isolate(&self) -> Result<&'static mut v8::OwnedIsolate, Error> {
        return unsafe {
            match self.isolate_ptr.as_mut() {
                Some(v) => Ok(v),
                None => Err(Error::UnitializedRuntime),
            }
        };
    }
    pub fn get_registry(&mut self, id: u64) -> &mut HashMap<String, Callable> {
        match self.registry.contains_key(&id) {
            true => self.registry.get_mut(&id).unwrap(),
            false => {
                self.registry.insert(id, HashMap::new());
                self.registry.get_mut(&id).unwrap()
            }
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.isolate_ptr) });
    }
}
