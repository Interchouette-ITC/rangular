use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

use crate::components::io_child::IoChildPanel;

include!(concat!(env!("OUT_DIR"), "/rangular/io_parent_view.rs"));

#[component]
pub fn IoParentPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let heading = RwSignal::new(String::from("Parent panel"));
    let label = RwSignal::new(String::from("Sounds"));
    let muted = RwSignal::new(false);

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            heading.set(String::from("Parent panel"));
            label.set(String::from("Sounds"));
            muted.set(false);
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        heading.set(format!("Parent (seed {n})"));
        label.set(format!("Channel {n}"));
        muted.set(n.is_multiple_of(2));
    });

    view! {
        <div class="io-parent-wrap">
            {io_parent_view(HostCell::new(IoParentHost { heading }))}
            <IoChildPanel label=label muted=muted />
        </div>
    }
}

struct IoParentHost {
    heading: RwSignal<String>,
}

impl Host for IoParentHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "heading").then(|| Value::Str(self.heading.get()))
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
