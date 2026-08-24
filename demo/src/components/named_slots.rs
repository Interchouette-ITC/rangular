use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/named_slots_view.rs"));

struct EmptyHost;

impl Host for EmptyHost {
    fn get(&self, _: &str) -> Option<Value> {
        None
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

#[component]
pub fn NamedSlotsPanel(tick: RwSignal<u32>) -> impl IntoView {
    let header = RwSignal::new(String::from("Named header slot"));
    let body = RwSignal::new(String::from("Default slot body"));

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        header.set(format!("Named header (seed {n})"));
        body.set(format!("Default body (seed {n})"));
    });

    named_slots_view(
        HostCell::new(EmptyHost),
        Box::new(move || {
            view! { <h1 class="header">{move || header.get()}</h1> }.into_any()
        }),
        Box::new(move || view! { <p>{move || body.get()}</p> }.into_any()),
    )
}
