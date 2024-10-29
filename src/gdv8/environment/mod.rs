use rusty_v8 as v8;

pub fn as_raw<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

pub fn unsafe_mut<T>(value: *mut T) -> &'static mut T {
    return unsafe { value.as_mut() }.unwrap();
}

pub type V8Isolate = v8::OwnedIsolate;
pub type V8HandleScope<'a> = v8::HandleScope<'a, ()>;
pub type V8Context<'a> = v8::Local<'a, v8::Context>;

pub struct V8Environment<'a> {
    pub isolate: *mut V8Isolate,
    pub scope: *mut V8HandleScope<'a>,
    pub context: *mut V8Context<'a>,
}

unsafe impl Send for V8Environment<'_> {}

impl V8Environment<'static> {
    pub fn new() -> Self {
        // Create a new Isolate and make it the current one.
        let isolate_raw = as_raw(v8::Isolate::new(v8::CreateParams::default()));
        let isolate = unsafe_mut(isolate_raw);

        // Create a stack-allocated handle scope.
        let handle_scope_raw = as_raw(v8::HandleScope::new(isolate));
        let handle_scope = unsafe_mut(handle_scope_raw);

        // Create a new context.
        let context_raw = as_raw(v8::Context::new(handle_scope));

        return Self {
            isolate: unsafe_mut(isolate_raw),
            context: unsafe_mut(context_raw),
            scope: unsafe_mut(handle_scope_raw),
        };
    }
    pub fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.scope));
            // drop(Box::from_raw(self.context));
            // drop(Box::from_raw(self.isolate));
        }
    }
}
