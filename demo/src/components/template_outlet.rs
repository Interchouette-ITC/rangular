use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/template_outlet_view.rs"));

const LABELS: [&str; 4] = ["Card", "Panel", "Tile", "Stamp"];

#[component]
pub fn TemplateOutletPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let label = RwSignal::new(String::from(LABELS[0]));

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        label.set(LABELS[(n as usize) % LABELS.len()].to_owned());
    });

    template_outlet_view(HostCell::new(TemplateOutletHost { label }))
}

struct TemplateOutletHost {
    label: RwSignal<String>,
}

impl Host for TemplateOutletHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "label").then(|| Value::Str(self.label.get()))
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
