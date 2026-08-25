use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/item_list_view.rs"));

const DEMO_ITEMS: [&str; 3] = [
    "Host exposes Vec<String>",
    "@for + $index / track item",
    "List rotates on Random",
];

#[component]
pub fn ItemListPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let title = RwSignal::new(String::from("item-list (@for)"));
    let items = RwSignal::new(DEMO_ITEMS.iter().map(|s| (*s).to_owned()).collect());

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        let offset = (n as usize) % DEMO_ITEMS.len();
        let rotated: Vec<String> = DEMO_ITEMS
            .iter()
            .cycle()
            .skip(offset)
            .take(DEMO_ITEMS.len())
            .copied()
            .map(str::to_string)
            .collect();
        items.set(rotated);
        title.set(format!("item-list (@for, {seed})"));
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
