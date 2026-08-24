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
pub fn NamedSlotsPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let header = RwSignal::new(String::from("Named header slot"));
    let body = RwSignal::new(String::from("Default slot body"));

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            header.set(String::from("Named header slot"));
            body.set(String::from("Default slot body"));
            return;
        }
        header.set(format!("Named header ({seed})"));
        body.set(format!("Default body ({seed})"));
    });

    named_slots_view(
        HostCell::new(EmptyHost),
        Box::new(move || {
            view! { <h1 class="header">{move || header.get()}</h1> }.into_any()
        }),
        Box::new(move || view! { <p>{move || body.get()}</p> }.into_any()),
    )
}
