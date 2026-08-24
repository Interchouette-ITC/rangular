use leptos::prelude::*;

use super::IoChildPanel;

#[component]
pub fn IoChildDemoPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let label = RwSignal::new(String::from("Feature flag"));
    let muted = RwSignal::new(false);

    Effect::new(move |_| {
        let seed = applied_seed.get();
        if seed.is_empty() {
            label.set(String::from("Feature flag"));
            muted.set(false);
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        label.set(format!("Channel {n}"));
        muted.set(n.is_multiple_of(2));
    });

    view! {
        <div class="io-child-line">
            <span class="io-child-line__title">{move || label.get()}</span>
            <span class="io-child-line__muted">{move || if muted.get() { "true" } else { "false" }}</span>
            <IoChildPanel label=label muted=muted />
        </div>
    }
}
