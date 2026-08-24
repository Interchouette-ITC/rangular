use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/pipes_view.rs"));

const LABELS: [&str; 4] = ["Hello", "rangular", "Ferris", "pipes"];

#[component]
pub fn PipesPanel(tick: RwSignal<u32>) -> impl IntoView {
    let label = RwSignal::new(String::from(LABELS[0]));
    let amount = RwSignal::new(42.5_f64);

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        label.set(LABELS[(n as usize) % LABELS.len()].to_owned());
        amount.set(10.0 + f64::from(n % 90));
    });

    pipes_view(HostCell::new(PipesHost { label, amount }))
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
