use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/two_way_view.rs"));

#[component]
pub fn TwoWayPanel(tick: RwSignal<u32>) -> impl IntoView {
    let seed = RwSignal::new(String::from("abc"));

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        seed.set(format!("seed-{n}"));
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

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
