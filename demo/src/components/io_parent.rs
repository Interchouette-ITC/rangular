use leptos::prelude::*;

use super::IoChildPanel;

#[component]
pub fn IoParentPanel(tick: RwSignal<u32>) -> impl IntoView {
    let heading = RwSignal::new(String::from("Alerts"));
    let child_label = RwSignal::new(String::from("Sounds"));
    let muted = RwSignal::new(true);

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        heading.set(format!("Alerts (seed {n})"));
        child_label.set(format!("Channel {n}"));
        muted.set(n.is_multiple_of(2));
    });

    view! {
        <section class="io-parent" aria-label="Parent child IO sample">
            <h2>{move || heading.get()}</h2>
            <IoChildPanel label=child_label muted=muted />
        </section>
    }
}
