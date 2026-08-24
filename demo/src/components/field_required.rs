use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{required, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/field_required_view.rs"));

#[component]
pub fn FieldRequiredPanel(tick: RwSignal<u32>) -> impl IntoView {
    let name = RwSignal::new(String::new());

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        if n.is_multiple_of(3) {
            name.set(String::new());
        } else {
            name.set(format!("User {n}"));
        }
    });

    field_required_view(HostCell::new(FieldRequiredHost { name }))
}

struct FieldRequiredHost {
    name: RwSignal<String>,
}

impl Host for FieldRequiredHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "name" => Some(Value::Str(self.name.get())),
            "nameInvalid" => Some(Value::Bool(required(&self.name.get()).is_some())),
            "nameError" => Some(Value::Str(
                required(&self.name.get()).unwrap_or("").to_owned(),
            )),
            _ => None,
        }
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        if name == "name" {
            if let Some(s) = value.as_str() {
                self.name.set(s.to_owned());
            }
        }
        Ok(())
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
