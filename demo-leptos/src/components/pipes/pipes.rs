use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

use crate::demo_pipes::demo_pipes;

include!(concat!(env!("OUT_DIR"), "/rangular/pipes_view.rs"));

const LABELS: [&str; 4] = ["Hello", "rangular", "Ferris", "pipes"];

#[component]
pub fn PipesPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let label = RwSignal::new(String::from(LABELS[0]));
    let amount = RwSignal::new(42.5_f64);

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        label.set(LABELS[(n as usize) % LABELS.len()].to_owned());
        amount.set(10.0 + f64::from(n % 90));
    });

    pipes_view(HostCell::with_pipes(
        PipesHost { label, amount },
        demo_pipes(),
    ))
}

struct PipesHost {
    label: RwSignal<String>,
    amount: RwSignal<f64>,
}

impl Host for PipesHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "label" => Some(Value::Str(self.label.get())),
            "amount" => Some(Value::Num(self.amount.get())),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
