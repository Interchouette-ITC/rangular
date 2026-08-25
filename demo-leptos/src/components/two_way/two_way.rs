use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/two_way_view.rs"));

#[component]
pub fn TwoWayPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let seed = RwSignal::new(String::from("abc"));

    Effect::new(move |_| {
        let value = applied_seed.get();
        if value.is_empty() {
            return;
        }
        seed.set(value);
    });

    two_way_view(HostCell::new(TwoWayHost { seed }))
}

struct TwoWayHost {
    seed: RwSignal<String>,
}

impl Host for TwoWayHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "seed").then(|| Value::Str(self.seed.get()))
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        if name == "seed" {
            if let Some(s) = value.as_str() {
                self.seed.set(s.to_owned());
            }
        }
        Ok(())
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "pushFromHost" {
            self.seed.set(String::from("host-push"));
        }
        Ok(Value::Unit)
    }
}
