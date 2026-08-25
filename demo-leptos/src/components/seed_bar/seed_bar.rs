use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{EventPayload, Host, HostError, Value};
use std::fmt::Write as _;

include!(concat!(env!("OUT_DIR"), "/rangular/seed_bar_view.rs"));

#[component]
pub fn SeedBarPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let seed = RwSignal::new(String::from("demo-seed"));
    let random_seq = RwSignal::new(0_u32);

    seed_bar_view(HostCell::new(SeedBarHost {
        seed,
        applied_seed,
        random_seq,
    }))
}

struct SeedBarHost {
    seed: RwSignal<String>,
    applied_seed: RwSignal<String>,
    random_seq: RwSignal<u32>,
}

impl SeedBarHost {
    fn generate_disabled(&self) -> bool {
        let draft = self.seed.get();
        let draft = draft.trim();
        draft.is_empty() || draft == self.applied_seed.get()
    }
}

impl Host for SeedBarHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.get())),
            "generateDisabled" => Some(Value::Bool(self.generate_disabled())),
            "randomDisabled" => Some(Value::Bool(false)),
            _ => None,
        }
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        if name == "seed" {
            if let Some(s) = value.as_str() {
                self.seed.set(s.to_owned());
            }
        }
        Ok(())
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        match name {
            "seedChange" => {
                if let Some(EventPayload::Input { value }) = args.first().and_then(Value::as_event)
                {
                    self.seed.set(value.clone());
                }
            }
            "onGenerate" => {
                apply_seed(&self.seed, self.applied_seed);
            }
            "onRandom" => {
                apply_random(&self.seed, self.applied_seed, self.random_seq);
            }
            _ => {}
        }
        Ok(Value::Unit)
    }
}

fn apply_seed(seed: &RwSignal<String>, applied_seed: RwSignal<String>) {
    let current = seed.get().trim().to_owned();
    if current.is_empty() {
        return;
    }
    applied_seed.set(current);
}

fn apply_random(
    seed: &RwSignal<String>,
    applied_seed: RwSignal<String>,
    random_seq: RwSignal<u32>,
) {
    random_seq.update(|n| {
        *n = n.wrapping_add(1);
        if *n == 0 {
            *n = 1;
        }
    });
    let next = random_seed_hex(random_seq.get_untracked());
    seed.set(next.clone());
    applied_seed.set(next);
}

fn random_seed_hex(seq: u32) -> String {
    let mut out = String::with_capacity(16);
    let mut state = seq.wrapping_add(0xA5A5_1234);
    for _ in 0..8 {
        state = state.wrapping_mul(0x9E37_79B9).wrapping_add(0x517C_C911);
        let b = (state >> 24) as u8;
        let _ = write!(out, "{b:02x}");
    }
    out
}
