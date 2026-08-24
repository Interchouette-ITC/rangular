use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};
use wasm_bindgen::JsCast;

include!(concat!(env!("OUT_DIR"), "/rangular/chrome_header_view.rs"));

#[component]
pub fn ChromeHeaderPanel(tick: RwSignal<u32>) -> impl IntoView {
    let muted = RwSignal::new(false);
    let enabled_count = RwSignal::new(2_u32);
    let total_count = RwSignal::new(5_u32);
    let paused_count = Memo::new(move |_| total_count.get().saturating_sub(enabled_count.get()));
    let count_label = Memo::new(move |_| {
        format!("{}/{}", enabled_count.get(), total_count.get())
    });

    Effect::new(move |_| {
        let _ = tick.get();
        let total = 3 + (js_random_u32() % 8);
        let enabled = 1 + (js_random_u32() % total);
        total_count.set(total);
        enabled_count.set(enabled);
    });

    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
            move |event: web_sys::Event| {
                let Some(key_event) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
                    return;
                };
                if key_event.key().eq_ignore_ascii_case("m") {
                    muted.update(|m| *m = !*m);
                }
            },
        );
        let listener = closure.as_ref().unchecked_ref();
        let _ = window.add_event_listener_with_callback("keydown", listener);
        closure.forget();
    });

    chrome_header_view(HostCell::new(ChromeHeaderHost {
        muted,
        count_label,
        enabled_count,
        total_count,
        paused_count,
    }))
}

struct ChromeHeaderHost {
    muted: RwSignal<bool>,
    count_label: Memo<String>,
    enabled_count: RwSignal<u32>,
    total_count: RwSignal<u32>,
    paused_count: Memo<u32>,
}

impl Host for ChromeHeaderHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "muted" => Some(Value::Bool(self.muted.get())),
            "countLabel" => Some(Value::Str(self.count_label.get())),
            "enabledCount" => Some(Value::Num(f64::from(self.enabled_count.get()))),
            "totalCount" => Some(Value::Num(f64::from(self.total_count.get()))),
            "pausedCount" => Some(Value::Num(f64::from(self.paused_count.get()))),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "toggleMute" {
            self.muted.update(|m| *m = !*m);
        }
        Ok(Value::Unit)
    }
}

fn js_random_u32() -> u32 {
    let n = js_sys::Math::random();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let scaled = (n * f64::from(u32::MAX)).round() as u32;
    scaled
}
