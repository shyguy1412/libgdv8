use crate::{
    error::Error,
    helper::{AsLocal, AsValue},
    Callable, Runtime, Value, V8_RUNTIME,
};
use godot::meta::{FromGodot, ToGodot};
use rusty_v8::{self as v8};

static mut CONTEXT_COUNT: u64 = 0;

pub struct Context {
    id: u64,
    handle_scope_ptr: *mut v8::HandleScope<'static, ()>,
    context_ptr: *mut v8::Local<'static, v8::Context>,
    context_scope_ptr: *mut v8::ContextScope<'static, v8::HandleScope<'static>>,
}

unsafe impl Send for Context {}

impl Context {
    pub fn new() -> Self {
        let guard = V8_RUNTIME.lock().unwrap();
        let runtime = guard.get_or_init(|| Runtime::new());

        let isolate = runtime
            .get_isolate()
            .expect("Isolate should always be ready");
        let handle_scope_ptr = Box::into_raw(Box::new(v8::HandleScope::new(isolate)));

        let context_ptr = Box::into_raw(Box::new(v8::Context::new(unsafe {
            handle_scope_ptr.as_mut().unwrap()
        })));

        let handle_scope = unsafe { handle_scope_ptr.as_mut().unwrap() };
        let context = unsafe { context_ptr.as_ref().unwrap() };

        let context_scope_ptr =
            Box::into_raw(Box::new(v8::ContextScope::new(handle_scope, *context)));

        return Self {
            id: unsafe {
                CONTEXT_COUNT += 1;
                CONTEXT_COUNT
            },
            handle_scope_ptr,
            context_ptr,
            context_scope_ptr,
        };
    }

    fn context_scope(
        &self,
    ) -> Result<&mut v8::ContextScope<'static, v8::HandleScope<'static>>, Error> {
        return match unsafe { self.context_scope_ptr.as_mut() } {
            Some(v) => Ok(v),
            None => Err(Error::InvalidContext),
        };
    }

    pub fn run_script(&self, source: &str) -> Result<v8::Local<'_, v8::Value>, Error> {
        let scope = self.context_scope()?;

        let scope = &mut v8::TryCatch::new(scope);

        let source = source.as_local(scope)?;
        let name = "name".as_local(scope)?.into();

        let result = v8::Script::compile(scope, source, None).and_then(|script| script.run(scope));

        return match result {
            Some(v) => Ok(v.clone()),
            None => Err(match scope.has_caught() {
                true => Error::Exception(
                    scope
                        .exception()
                        .unwrap()
                        .to_object(scope)
                        .unwrap()
                        .get(scope, name)
                        .unwrap()
                        .to_rust_string_lossy(scope),
                ),
                false => Error::None,
            }),
        };
    }

    pub fn register_callable(&self, identifier: &str, callable: Callable) -> Result<(), Error> {
        let function = |scope: &mut v8::HandleScope<'_>,
                        args: v8::FunctionCallbackArguments,
                        mut retval: v8::ReturnValue| {
            let scope = &mut v8::TryCatch::new(scope);
            let mut weak: Vec<Value> = vec![];

            for i in 0..args.length() {
                // godot_print!("PARSING ARG {}", i);
                let current_arg = args.get(i).as_value(scope);
                // godot_print!("current arg: {current_arg}");
                // let value = current_arg.to_rust_string_lossy(scope);
                weak.push(current_arg);
            }

            let data = args.data().unwrap().to_object(scope).unwrap();
            let context_id_key = v8::String::new(scope, "contextId").unwrap();
            let identifier_key = v8::String::new(scope, "identifier").unwrap();
            let name = v8::String::new(scope, "name").unwrap();

            let context_id = data
                .get(scope, context_id_key.into())
                .unwrap()
                .number_value(scope)
                .unwrap()
                .to_be_bytes();
            let context_id = u64::from_be_bytes(context_id);

            let callback_identifier = data
                .get(scope, identifier_key.into())
                .unwrap()
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);

            let mut guard = V8_RUNTIME.lock().unwrap();
            let runtime = guard
                .get_mut()
                .expect("exposed callable run without runtime");

            let registry = runtime.get_registry(context_id);
            let callback = registry
                .get(&callback_identifier)
                .expect("callbacks should be registered at this point");

            let godot_args = godot::builtin::Array::from_iter(weak.iter().map(|v| v.to_godot()));

            let result = match callback {
                Callable::Godot(callable) => {
                    // godot_print!("callable: {callable}, {}", callable.is_valid());
                    let result = callable.callv(&godot::builtin::VariantArray::from(godot_args));
                    let result = Value::from_godot(result);
                    result
                }
                Callable::Closure(v) => v(weak),
            };
            
            retval.set(result.as_local(scope));

            let error = match scope.has_caught() {
                true => Error::Exception(
                    scope
                        .exception()
                        .unwrap()
                        .to_object(scope)
                        .unwrap()
                        .get(scope, name.into())
                        .unwrap()
                        .to_rust_string_lossy(scope),
                ),
                false => Error::None,
            };
        };

        let mut guard = V8_RUNTIME.lock().unwrap();
        let runtime = match guard.get_mut() {
            Some(v) => v,
            None => return Err(Error::UnitializedRuntime),
        };

        match &callable {
            Callable::Godot(callable) => callable.callv(&godot::builtin::VariantArray::from(
                godot::builtin::Array::new(),
            )),
            Callable::Closure(_) => todo!(),
        };

        let scope = self.context_scope()?;
        let registry = runtime.get_registry(self.id);
        registry.insert(identifier.to_string(), callable);

        let identifier = identifier.as_local(scope)?;
        let id = v8::Number::new(scope, f64::from_be_bytes(self.id.to_be_bytes()));
        let callback_data = v8::Object::new(scope);

        let context_id_key = v8::String::new(scope, "contextId").unwrap();
        let identifier_key = v8::String::new(scope, "identifier").unwrap();

        callback_data.set(scope, context_id_key.into(), id.into());
        callback_data.set(scope, identifier_key.into(), identifier.into());

        let function = v8::FunctionBuilder::<v8::FunctionTemplate>::new(function)
            .data(callback_data.into())
            .build(scope)
            .get_function(scope)
            .unwrap();

        scope
            .get_current_context()
            .global(scope)
            .set(scope, identifier.into(), function.into());

        return Ok(());
    }

    pub fn to_rust_string_lossy(&self, value: v8::Local<rusty_v8::Value>) -> Result<String, Error> {
        let context_scope = self.context_scope()?;
        Ok(value.to_rust_string_lossy(context_scope))
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.context_scope_ptr));
            drop(Box::from_raw(self.context_ptr));
            drop(Box::from_raw(self.handle_scope_ptr));
        }
    }
}
