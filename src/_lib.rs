mod gdv8;

use godot::{classes::notify::NodeNotification, prelude::*};

struct GodotV8Extension;
#[gdextension]
unsafe impl ExtensionLibrary for GodotV8Extension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct JSNode {
    base: Base<Node>,
    context: gdv8::Context,
}

#[godot_api]
impl JSNode {
    #[func]
    fn run_script(&mut self, script: String) -> gdv8::WeakType {
        let value = self.runtime.run_script(&script);

        return match value {
            Ok(v) => match self.runtime.to_rust_string_lossy(v) {
                Ok(v) => gdv8::String(v),
                Err(_) => gdv8::WeakType::Undefined,
            },
            Err(_) => gdv8::WeakType::Undefined,
        };
    }
}

#[godot_api]
impl INode for JSNode {
    fn init(base: Base<Node>) -> Self {
        let mut runtime = gdv8::Runtime::prepare();
        if runtime.init().is_err() {
            panic!("Could not create V8 runtime")
        };
        Self { base, runtime }
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {}
    }
}
