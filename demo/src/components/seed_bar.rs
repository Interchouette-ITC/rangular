use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/seed_bar_view.rs"));

#[component]
pub fn SeedBarPanel() -> impl IntoView {
    let seed = RwSignal::new(String::from("demo-seed"));
    let status = RwSignal::new(String::new());

    seed_bar_view(HostCell::new(SeedBarHost {
        seed,
        status,
        generate_disabled: RwSignal::new(false),
        random_disabled: RwSignal::new(false),
    }))
}

struct SeedBarHost {
    seed: RwSignal<String>,
    status: RwSignal<String>,
    generate_disabled: RwSignal<bool>,
    random_disabled: RwSignal<bool>,
}

impl SeedBarHost {
    fn read_input(args: &[Value]) -> Option<String> {
        args.first()
            .and_then(|v| v.as_event())
            .and_then(|payload| match payload {
                EventPayload::Input { value } => Some(value.clone()),
                _ => None,
            })
    }
}

impl Host for SeedBarHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.get())),
            "generateDisabled" => Some(Value::Bool(self.generate_disabled.get())),
            "randomDisabled" => Some(Value::Bool(self.random_disabled.get())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "seedChange" => {
                if let Some(next) = Self::read_input(args) {
                    self.seed.set(next);
                }
                Ok(Value::Unit)
            }
            "onGenerate" => {
                self.status.set(format!("Generate clicked ({})", self.seed.get()));
                Ok(Value::Unit)
            }
            "onRandom" => {
                let next = format!("{:08x}", js_random_u32());
                self.seed.set(next.clone());
                self.status.set(format!("Random seed → {next}"));
                Ok(Value::Unit)
            }
            _ => Ok(Value::Unit),
        }
    }
}

fn js_random_u32() -> u32 {
    let n = js_sys::Math::random();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let scaled = (n * f64::from(u32::MAX)).round() as u32;
    scaled
}
