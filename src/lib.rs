use std::sync::{LazyLock, Mutex};

use godot::{classes::notify::Node3DNotification, prelude::*};
use rusty_v8 as v8;
struct GodotV8Extension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotV8Extension {}

// fn as_static<T>(value: T) -> &'static T {
//     Box::leak(Box::new(value))
// }

// fn as_static_mut<T>(value: T) -> &'static mut T {
//     Box::leak(Box::new(value))
// }

fn as_raw<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

fn unsafe_mut<T>(value: *mut T) -> &'static mut T {
    return unsafe { value.as_mut() }.unwrap();
}

fn unsafe_ref<T>(value: *mut T) -> &'static T {
    return unsafe { value.as_ref() }.unwrap();
}

static V8_IS_INITIALIZED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

type V8Isolate = *mut v8::OwnedIsolate;
type V8HandleScope<'a> = *mut v8::HandleScope<'a, ()>;
type V8Context<'a> = *mut v8::Local<'a, v8::Context>;
type V8Scope<'a> = *mut v8::ContextScope<'a, v8::HandleScope<'a>>;

struct V8Environment<'a> {
    isolate: V8Isolate,
    handle_scope: V8HandleScope<'a>,
    context: V8Context<'a>,
    context_scope: V8Scope<'a>,
}

impl V8Environment<'_> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.context_scope));
            drop(Box::from_raw(self.handle_scope));
            drop(Box::from_raw(self.context));
            drop(Box::from_raw(self.isolate));
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct Test {
    base: Base<Node3D>,
    environment: V8Environment<'static>,
}

#[godot_api]
impl Test {
    #[func]
    fn run_script(&mut self, script: String) {
        godot_print!("init_run");
        ScriptController::run_script(self, &script);
    }

}

#[godot_api]
impl INode3D for Test {
    fn init(node3d: Base<Node3D>) -> Self {
        godot_print!("HELLO WORLD");

        let mut guard = V8_IS_INITIALIZED.lock().unwrap();

        if !guard.get_property() {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            *guard = true;
        }

        Self {
            base: node3d,
            environment: Test::create_environment(),
        }
    }

    fn on_notification(&mut self, what: Node3DNotification){
        if what == Node3DNotification::PREDELETE {
            self.environment.drop();
        }
    }

}

trait ScriptController {
    fn run_script(&mut self, source: &str);
    fn create_environment() -> V8Environment<'static>;
}

impl ScriptController for Test {
    fn create_environment() -> V8Environment<'static> {
        // Create a new Isolate and make it the current one.
        let isolate_raw = as_raw(v8::Isolate::new(v8::CreateParams::default()));
        let isolate = unsafe_mut(isolate_raw);

        // Create a stack-allocated handle scope.
        let handle_scope_raw = as_raw(v8::HandleScope::new(isolate));
        let handle_scope = unsafe_mut(handle_scope_raw);

        // Create a new context.
        let context_raw = as_raw(v8::Context::new(handle_scope));
        let context = unsafe_ref(context_raw);

        // Enter the context for compiling and running the hello world script.
        let scope_raw = as_raw(v8::ContextScope::new(handle_scope, *context));

        return V8Environment {
            context_scope: scope_raw,
            isolate: isolate_raw,
            context: context_raw,
            handle_scope: handle_scope_raw,
        };
    }

    fn run_script(&mut self, source: &str) {

        // Create a string containing the JavaScript source code.
        godot_print!("pre scope");
        let scope = unsafe_mut(self.environment.context_scope);
        godot_print!("post scope");

        let code = v8::String::new(scope, source).unwrap();

        // Compile the source code.
        let script = v8::Script::compile(scope, code, None).unwrap();

        // Run the script to get the result.
        let result = match script.run(scope) {
            Some(val) => val.to_rust_string_lossy(scope),
            None => "undefined".to_string(),
        };
        godot_print!("{}: {}", source, result);
        drop(result);
    }
}
