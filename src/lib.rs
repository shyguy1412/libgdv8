mod gdv8;
mod helpers;

use godot::{classes::notify::Node3DNotification, prelude::*};
use gdv8::environment::{StaticV8Environment, UnwrapV8, V8Environment};
use rusty_v8 as v8;

struct GodotV8Extension;
#[gdextension]
unsafe impl ExtensionLibrary for GodotV8Extension {}

static V8_ENVIRONMENT:StaticV8Environment = V8Environment::prepare_static();

#[derive(GodotClass)]
#[class(base=Node3D)]
struct Test {
    base: Base<Node3D>,
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

        let mut v8 = V8_ENVIRONMENT.lock().unwrap();

        if v8.is_none() {
            *v8 = Some(V8Environment::new());
        }

        Self { base: node3d }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        if what == Node3DNotification::PREDELETE {
            // self.environment.drop();
        }
    }
}

trait ScriptController {
    fn run_script(&mut self, source: &str) -> Option<v8::Local<'_, v8::Value>>;
}

impl ScriptController for Test {
    fn run_script(&mut self, source: &str) -> Option<v8::Local<'_, v8::Value>> {

        // Create a string containing the JavaScript source code.
        let v8_lock = &mut V8_ENVIRONMENT.lock().unwrap();
        let v8 = v8_lock.unwrap_v8();
        let scope = v8.context_scope;

        let code = v8::String::new(scope, source).unwrap();

        // Compile the source code.
        let script = v8::Script::compile(scope, code, None).unwrap();

        // Run the script to get the result.
        let result = script.run(scope);
        return result;
    }
}
