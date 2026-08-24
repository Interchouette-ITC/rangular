use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/seed_bar_view.rs"));

#[component]
pub fn SeedBarPanel(tick: RwSignal<u32>, on_randomize: Callback<()>) -> impl IntoView {
    let seed = RwSignal::new(String::from("demo-seed"));
    let busy = RwSignal::new(false);
    let worker_ready = RwSignal::new(true);
    let generated_for = RwSignal::new(String::new());

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        seed.set(format!("seed-{n}"));
        generated_for.set(String::new());
    });

    seed_bar_view(HostCell::new(SeedBarHost {
        seed,
        busy,
        worker_ready,
        generated_for,
        on_randomize,
    }))
}

struct SeedBarHost {
    seed: RwSignal<String>,
    busy: RwSignal<bool>,
    worker_ready: RwSignal<bool>,
    generated_for: RwSignal<String>,
    on_randomize: Callback<()>,
}

impl SeedBarHost {
    fn generate_disabled(&self) -> bool {
        !self.worker_ready.get()
            || self.busy.get()
            || (!self.generated_for.get_untracked().is_empty()
                && self.generated_for.get_untracked() == self.seed.get_untracked())
    }

    fn random_disabled(&self) -> bool {
        !self.worker_ready.get() || self.busy.get()
    }
}

impl Host for SeedBarHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.get())),
            "generateDisabled" => Some(Value::Bool(self.generate_disabled())),
            "randomDisabled" => Some(Value::Bool(self.random_disabled())),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "seedChange" => {
                if let Some(EventPayload::Input { value }) = args.first().and_then(|v| v.as_event())
                {
                    self.seed.set(value.clone());
                }
            }
            "onGenerate" => {
                self.generated_for.set(self.seed.get_untracked());
            }
            "onRandom" => {
                self.on_randomize.run(());
            }
            _ => {}
        }
        Ok(Value::Unit)
    }
}
