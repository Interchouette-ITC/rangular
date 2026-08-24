use leptos::prelude::*;
use std::fmt::Write as _;

#[component]
pub fn SeedBarPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let seed = RwSignal::new(String::from("demo-seed"));
    let random_seq = RwSignal::new(0_u32);

    view! {
        <div class="seed-bar-wrap" id="seed-bar">
            <section class="seed-bar" aria-label="Seed controls">
                <label for="demo-seed-input">"Seed"</label>
                <input
                    id="demo-seed-input"
                    type="text"
                    bind:value=seed
                    spellcheck="false"
                    autocomplete="off"
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            apply_seed(&seed, applied_seed);
                        }
                    }
                />
                <button
                    class="btn btn-primary"
                    type="button"
                    prop:disabled=move || {
                        generate_disabled(&seed.get(), &applied_seed.get())
                    }
                    on:click=move |_| apply_seed(&seed, applied_seed)
                >
                    "Generate"
                </button>
                <button
                    class="btn btn-secondary"
                    type="button"
                    on:click=move |_| apply_random(&seed, applied_seed, random_seq)
                >
                    "Random"
                </button>
            </section>
        </div>
    }
}

fn generate_disabled(seed: &str, applied: &str) -> bool {
    let draft = seed.trim();
    draft.is_empty() || draft == applied
}

fn apply_seed(seed: &RwSignal<String>, applied_seed: RwSignal<String>) {
    let current = seed.get().trim().to_owned();
    if current.is_empty() {
        return;
    }
    applied_seed.set(current);
}

fn apply_random(seed: &RwSignal<String>, applied_seed: RwSignal<String>, random_seq: RwSignal<u32>) {
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
        state = state
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(0x517C_C911);
        let b = (state >> 24) as u8;
        let _ = write!(out, "{b:02x}");
    }
    out
}
