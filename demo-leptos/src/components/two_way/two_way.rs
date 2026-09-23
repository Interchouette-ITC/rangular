use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/two_way_view.rs"));

#[component]
pub fn TwoWayPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let seed = RwSignal::new(String::from("abc"));
    let on = RwSignal::new(false);

    Effect::new(move |_| {
        let value = applied_seed.get();
        if value.is_empty() {
            return;
        }
        seed.set(value);
    });

    two_way_view(HostCell::new(TwoWayHost { seed, on }))
}

struct TwoWayHost {
    seed: RwSignal<String>,
    on: RwSignal<bool>,
}

impl Host for TwoWayHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.get())),
            "on" => Some(Value::Bool(self.on.get())),
            _ => None,
        }
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        match name {
            "seed" => {
                if let Some(s) = value.as_str() {
                    self.seed.set(s.to_owned());
                }
            }
            "on" => {
                if let Some(b) = value.as_bool() {
                    self.on.set(b);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "pushFromHost" {
            self.seed.set(String::from("host-push"));
            self.on.set(true);
        }
        Ok(Value::Unit)
    }
}
