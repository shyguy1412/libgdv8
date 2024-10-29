mod gdv8;
mod helpers;

use godot::{
    classes::notify::NodeNotification, prelude::*
};

struct GodotV8Extension;
#[gdextension]
unsafe impl ExtensionLibrary for GodotV8Extension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct JSNode {
    base: Base<Node>,
    runtime: gdv8::Runtime,
}

#[godot_api]
impl JSNode {
    #[func]
    fn run_script(&mut self, script: String) -> gdv8::WeakType {
        let value = match self.runtime.run_script(&script).as_deref() {
            Some(_) => gdv8::String("defined"),
            None => gdv8::String("undefined"),
        };
        return value;
    }
}

#[godot_api]
impl INode for JSNode {
    fn init(base: Base<Node>) -> Self {
        let mut runtime = gdv8::Runtime::prepare();
        runtime.init();
        Self { base, runtime }
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {
        }
    }
}
