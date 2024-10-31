use crate::gdv8::error::Error;
use godot::global::godot_print;
use rusty_v8::{self as v8};

pub struct V8Environment {
    isolate_ptr: *mut v8::OwnedIsolate,
    handle_scope_ptr: *mut v8::HandleScope<'static, ()>,
    context_ptr: *mut v8::Local<'static, v8::Context>,
    context_scope_ptr: *mut v8::ContextScope<'static, v8::HandleScope<'static>>,
}

unsafe impl Send for V8Environment {}

impl V8Environment {
    pub fn new() -> Result<Self, Error> {
        let isolate_ptr = Box::into_raw(Box::new(v8::Isolate::new(v8::CreateParams::default())));

        let handle_scope_ptr = unsafe { isolate_ptr.as_mut() }
            .and_then(|isolate| Some(Box::into_raw(Box::new(v8::HandleScope::new(isolate)))));

        let context_ptr = handle_scope_ptr
            .and_then(|handle_scope_ptr| unsafe { handle_scope_ptr.as_mut() })
            .and_then(|scope| Some(Box::into_raw(Box::new(v8::Context::new(scope)))));

        let handle_scope_ptr = match handle_scope_ptr {
            Some(v) => Ok(v),
            None => Err(Error::ScopePointerAllocationFailed),
        }?;

        let context_ptr = match context_ptr {
            Some(v) => Ok(v),
            None => Err(Error::ContextAllocationFailed),
        }?;

        let handle_scope = unsafe { handle_scope_ptr.as_mut().unwrap() };

        let context = unsafe { context_ptr.as_ref().unwrap() };

        let context_scope_ptr =
            Box::into_raw(Box::new(v8::ContextScope::new(handle_scope, *context)));

        return Ok(Self {
            isolate_ptr,
            handle_scope_ptr,
            context_ptr,
            context_scope_ptr,
        });
    }

    pub fn context_scope<'a>(
        &'a self,
    ) -> Option<&mut v8::ContextScope<'static, v8::HandleScope<'static>>> {
        return unsafe { self.context_scope_ptr.as_mut() };
    }
}

impl Drop for V8Environment {
    fn drop(&mut self) {
        godot_print!("dropped env");
        unsafe {
            drop(Box::from_raw(self.context_scope_ptr));
            drop(Box::from_raw(self.context_ptr));
            drop(Box::from_raw(self.handle_scope_ptr));
            drop(Box::from_raw(self.isolate_ptr));
        }
    }
}
