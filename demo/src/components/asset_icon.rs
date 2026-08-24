use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/asset_icon_view.rs"));

const CRAB: &str = "\u{1F980}\u{FE0E}";

#[component]
pub fn AssetIconPanel(accent: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="asset-icon-panel" style=move || format!("--demo-accent: {};", accent.get())>
            {asset_icon_view(HostCell::new(AssetIconHost {
                src: RwSignal::new(String::new()),
                letter: RwSignal::new(String::from(CRAB)),
                letter_fallback: RwSignal::new(true),
                size: RwSignal::new(String::from("md")),
                fallback_letter: CRAB,
            }))}
        </div>
    }
}

struct AssetIconHost {
    src: RwSignal<String>,
    letter: RwSignal<String>,
    letter_fallback: RwSignal<bool>,
    size: RwSignal<String>,
    fallback_letter: &'static str,
}

impl Host for AssetIconHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "src" => Some(Value::Str(self.src.get())),
            "letter" => Some(Value::Str(self.letter.get())),
            "letterFallback" => Some(Value::Bool(self.letter_fallback.get())),
            "size" => Some(Value::Str(self.size.get())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "onError" {
            self.src.set(String::new());
            self.letter.set(String::from(self.fallback_letter));
            self.letter_fallback.set(true);
        }
        Ok(Value::Unit)
    }
}
