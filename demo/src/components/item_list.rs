use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/item_list_view.rs"));

const FIXTURE_NAMES: [&str; 14] = [
    "chrome-header",
    "color-field",
    "item-list",
    "asset-icon",
    "layout-shell",
    "named-slots",
    "io-parent",
    "pipes",
    "two-way",
    "field-required",
    "event-payload",
    "template-outlet",
    "seed-bar",
    "io-child",
];

#[component]
pub fn ItemListPanel(tick: RwSignal<u32>) -> impl IntoView {
    let title = RwSignal::new(String::from("Fixture components"));
    let items = RwSignal::new(FIXTURE_NAMES.iter().map(|s| (*s).to_owned()).collect());

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        let offset = (n as usize) % FIXTURE_NAMES.len();
        let rotated: Vec<String> = FIXTURE_NAMES
            .iter()
            .cycle()
            .skip(offset)
            .take(FIXTURE_NAMES.len())
            .copied()
            .map(str::to_string)
            .collect();
        items.set(rotated);
        title.set(format!("Fixture components (seed {n})"));
    });

    item_list_view(HostCell::new(ItemListHost { title, items }))
}

struct ItemListHost {
    title: RwSignal<String>,
    items: RwSignal<Vec<String>>,
}

impl Host for ItemListHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "title" => Some(Value::Str(self.title.get())),
            "items" => Some(Value::List(
                self.items
                    .get()
                    .into_iter()
                    .map(Value::Str)
                    .collect(),
            )),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
