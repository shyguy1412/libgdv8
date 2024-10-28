use rusty_v8 as v8;
use std::sync::{LazyLock, Mutex, MutexGuard};

pub fn as_raw<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

pub fn unsafe_mut<T>(value: *mut T) -> &'static mut T {
    return unsafe { value.as_mut() }.unwrap();
}

pub trait UnwrapV8 {
    fn unwrap_v8(&mut self) -> V8Environment<'static>;
}

impl UnwrapV8 for MutexGuard<'_, Option<V8Environment<'_>>> {
    fn unwrap_v8(&mut self) -> V8Environment<'static> {
        todo!()
    }
}

pub type V8Isolate = v8::OwnedIsolate;
pub type V8HandleScope<'a> = v8::HandleScope<'a, ()>;
pub type V8Context<'a> = v8::Local<'a, v8::Context>;
pub type V8Scope<'a> = v8::ContextScope<'a, v8::HandleScope<'a>>;
pub type StaticV8Environment = LazyLock<Mutex<Option<V8Environment<'static>>>>;

pub struct V8Environment<'a> {
    pub isolate: &'a mut V8Isolate,
    pub handle_scope: &'a mut V8HandleScope<'a>,
    pub context: &'a mut V8Context<'a>,
    pub context_scope: &'a mut V8Scope<'a>,
}

unsafe impl Send for V8Environment<'_> {}

impl V8Environment<'static> {
    pub const fn prepare_static() -> StaticV8Environment {
        return LazyLock::new(|| Mutex::new(None));
    }
    pub fn new() -> Self {
        // Create a new Isolate and make it the current one.
        let isolate_raw = as_raw(v8::Isolate::new(v8::CreateParams::default()));
        let isolate = unsafe_mut(isolate_raw);

        // Create a stack-allocated handle scope.
        let handle_scope_raw = as_raw(v8::HandleScope::new(isolate));
        let handle_scope = unsafe_mut(handle_scope_raw);

        // Create a new context.
        let context_raw = as_raw(v8::Context::new(handle_scope));
        let context = unsafe_mut(context_raw);

        // Enter the context for compiling and running the hello world script.
        let context_scope_raw = as_raw(v8::ContextScope::new(handle_scope, *context));
        let context_scope = unsafe_mut(context_scope_raw);

        return Self {
            context_scope,
            isolate: unsafe_mut(isolate_raw),
            context,
            handle_scope: unsafe_mut(handle_scope_raw),
        };
    }
}
