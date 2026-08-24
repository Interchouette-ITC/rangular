use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/event_payload_view.rs"));

#[component]
pub fn EventPayloadPanel(tick: RwSignal<u32>) -> impl IntoView {
    let draft = RwSignal::new(String::from("draft"));
    let icon_src = RwSignal::new(String::from("/logo-256.png"));
    let status = RwSignal::new(String::new());

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        draft.set(format!("tick-{n}"));
        icon_src.set(if n.is_multiple_of(4) {
            "/missing.png".into()
        } else {
            "/logo-256.png".into()
        });
        status.set(String::new());
    });

    event_payload_view(HostCell::new(EventPayloadHost {
        draft,
        icon_src,
        status,
    }))
}

struct EventPayloadHost {
    draft: RwSignal<String>,
    icon_src: RwSignal<String>,
    status: RwSignal<String>,
}

impl Host for EventPayloadHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "draft" => Some(Value::Str(self.draft.get())),
            "iconSrc" => Some(Value::Str(self.icon_src.get())),
            "status" => Some(Value::Str(self.status.get())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "onInput" => {
                if let Some(EventPayload::Input { value }) = args.first().and_then(|v| v.as_event())
                {
                    self.draft.set(value.clone());
                    self.status.set(format!("input: {value}"));
                }
            }
            "onClick" => {
                self.status.set("clicked".into());
            }
            "onError" => {
                self.icon_src.set(String::new());
                self.status.set("image error".into());
            }
            _ => {}
        }
        Ok(Value::Unit)
    }
}
