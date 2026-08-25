use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{required, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/field_required_view.rs"));

#[component]
pub fn FieldRequiredPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let name_dirty = RwSignal::new(false);

    Effect::new(move |_| {
        let seed = applied_seed.get();
        name_dirty.set(false);
        if seed.is_empty() {
            name.set(String::new());
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        if n.is_multiple_of(3) {
            name.set(String::new());
        } else {
            name.set(format!("User {seed}"));
        }
    });

    field_required_view(HostCell::new(FieldRequiredHost { name, name_dirty }))
}

struct FieldRequiredHost {
    name: RwSignal<String>,
    name_dirty: RwSignal<bool>,
}

impl Host for FieldRequiredHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "name" => Some(Value::Str(self.name.get())),
            "nameDirty" => Some(Value::Bool(self.name_dirty.get())),
            "nameInvalid" => Some(Value::Bool(required(&self.name.get()).is_some())),
            "nameError" => Some(Value::Str(
                required(&self.name.get()).unwrap_or("").to_owned(),
            )),
            _ => None,
        }
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        if name == "name" {
            self.name_dirty.set(true);
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
