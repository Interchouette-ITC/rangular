use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/io_child_view.rs"));

#[component]
pub fn IoChildPanel(label: RwSignal<String>, muted: RwSignal<bool>) -> impl IntoView {
    io_child_view(HostCell::new(IoChildHost { label, muted }))
}

struct IoChildHost {
    label: RwSignal<String>,
    muted: RwSignal<bool>,
}

impl Host for IoChildHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "label" => Some(Value::Str(self.label.get())),
            "muted" => Some(Value::Bool(self.muted.get())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "muteToggle" {
            self.muted.update(|m| *m = !*m);
        }
        Ok(Value::Unit)
    }
}
