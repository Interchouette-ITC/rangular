use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/event_payload_view.rs"));

const LOGO_SRC: &str = "/logo-256.png";
const BROKEN_SRC: &str = "/missing.png";

#[component]
pub fn EventPayloadPanel() -> impl IntoView {
    let draft = RwSignal::new(String::new());
    let icon_src = RwSignal::new(String::from(LOGO_SRC));
    let img_broken = RwSignal::new(false);
    let payload_label = RwSignal::new(String::from("—"));

    event_payload_view(HostCell::new(EventPayloadHost {
        draft,
        icon_src,
        img_broken,
        payload_label,
    }))
}

struct EventPayloadHost {
    draft: RwSignal<String>,
    icon_src: RwSignal<String>,
    img_broken: RwSignal<bool>,
    payload_label: RwSignal<String>,
}

impl EventPayloadHost {
    fn show_payload(&self, payload: &EventPayload) {
        self.payload_label.set(payload.demo_label());
        if let EventPayload::Input { value } = payload {
            self.draft.set(value.clone());
        }
    }
}

impl Host for EventPayloadHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "draft" => Some(Value::Str(self.draft.get())),
            "iconSrc" => Some(Value::Str(self.icon_src.get())),
            "imgActionLabel" => Some(Value::Str(if self.img_broken.get() {
                "fix".into()
            } else {
                "break".into()
            })),
            "payloadLabel" => Some(Value::Str(self.payload_label.get())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "onInput" | "onClick" | "onError" | "onLoad" => {
                if let Some(payload) = args.first().and_then(Value::as_event) {
                    if matches!(payload, EventPayload::Error) {
                        self.img_broken.set(true);
                    }
                    if matches!(payload, EventPayload::Load) {
                        self.img_broken.set(false);
                    }
                    self.show_payload(payload);
                }
            }
            "onToggleImg" => {
                if self.img_broken.get() {
                    self.icon_src.set(String::from(LOGO_SRC));
                } else {
                    self.icon_src.set(String::from(BROKEN_SRC));
                }
            }
            _ => {}
        }
        Ok(Value::Unit)
    }
}
