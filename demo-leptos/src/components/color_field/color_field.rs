use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};
use wasm_bindgen::JsCast;

include!(concat!(env!("OUT_DIR"), "/rangular/color_field_view.rs"));

const DEFAULT_COLOR: &str = "#ce422b";
const SWATCHES: [&str; 12] = [
    "#ce422b", "#d81b60", "#8e24aa", "#5e35b1", "#3949ab", "#1e88e5", "#039be5", "#00acc1",
    "#00897b", "#43a047", "#fbc02d", "#fb8c00",
];

#[component]
pub fn ColorFieldPanel(applied_seed: RwSignal<String>, accent: RwSignal<String>) -> impl IntoView {
    let hex_draft = RwSignal::new(accent.get_untracked());
    let palette_open = RwSignal::new(false);
    let has_override = RwSignal::new(accent.get_untracked() != DEFAULT_COLOR);
    let anchor_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed) as usize;
        let idx = n % SWATCHES.len();
        apply_color(SWATCHES[idx], accent, hex_draft, has_override);
    });

    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
            move |ev: web_sys::Event| {
                if !palette_open.get() {
                    return;
                }
                let Some(anchor) = anchor_ref.get() else {
                    return;
                };
                let Some(target) = ev.target() else {
                    return;
                };
                let Ok(node) = target.dyn_into::<web_sys::Node>() else {
                    return;
                };
                if !anchor.contains(Some(&node)) {
                    palette_open.set(false);
                }
            },
        );
        let listener = closure.as_ref().unchecked_ref();
        let _ = document.add_event_listener_with_callback("click", listener);
        closure.forget();
    });

    view! {
        <div node_ref=anchor_ref>
            {color_field_view(HostCell::new(ColorFieldHost {
                label: "Accent color".into(),
                accent,
                hex_draft,
                input_id: "demo-accent".into(),
                palette_open,
                has_override,
                swatches: SWATCHES.iter().map(|s| (*s).to_owned()).collect(),
            }))}
        </div>
    }
}

struct ColorFieldHost {
    label: String,
    accent: RwSignal<String>,
    hex_draft: RwSignal<String>,
    input_id: String,
    palette_open: RwSignal<bool>,
    has_override: RwSignal<bool>,
    swatches: Vec<String>,
}

impl Host for ColorFieldHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "label" => Some(Value::Str(self.label.clone())),
            "value" => Some(Value::Str(self.accent.get())),
            "hexDraft" => Some(Value::Str(self.hex_draft.get())),
            "inputId" => Some(Value::Str(self.input_id.clone())),
            "paletteOpen" => Some(Value::Bool(self.palette_open.get())),
            "hasOverride" => Some(Value::Bool(self.has_override.get())),
            "swatches" => Some(Value::List(
                self.swatches.iter().cloned().map(Value::Str).collect(),
            )),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "isSwatchSelected" => {
                let hit = args
                    .first()
                    .and_then(Value::as_str)
                    .is_some_and(|swatch| swatch == self.accent.get());
                Ok(Value::Bool(hit))
            }
            "togglePalette" => {
                self.palette_open.update(|open| *open = !*open);
                Ok(Value::Unit)
            }
            "onReset" => {
                apply_color(DEFAULT_COLOR, self.accent, self.hex_draft, self.has_override);
                Ok(Value::Unit)
            }
            "onSwatch" => {
                if let Some(swatch) = args.first().and_then(Value::as_str) {
                    apply_color(swatch, self.accent, self.hex_draft, self.has_override);
                }
                Ok(Value::Unit)
            }
            "onColorInput" | "onHexInput" => {
                if let Some(EventPayload::Input { value }) = args.first().and_then(|v| v.as_event())
                {
                    apply_color(value, self.accent, self.hex_draft, self.has_override);
                }
                Ok(Value::Unit)
            }
            _ => Ok(Value::Unit),
        }
    }
}

fn apply_color(
    raw: &str,
    accent: RwSignal<String>,
    hex_draft: RwSignal<String>,
    has_override: RwSignal<bool>,
) {
    let normalized = normalize_hex(raw);
    has_override.set(normalized != DEFAULT_COLOR);
    accent.set(normalized.clone());
    hex_draft.set(normalized);
}

fn normalize_hex(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return DEFAULT_COLOR.to_owned();
    }
    let with_hash = if trimmed.starts_with('#') {
        trimmed.to_owned()
    } else {
        format!("#{trimmed}")
    };
    if with_hash.len() == 7 {
        with_hash
    } else {
        DEFAULT_COLOR.to_owned()
    }
}
