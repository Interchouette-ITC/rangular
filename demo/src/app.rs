use leptos::prelude::*;

use crate::components::{
    AssetIconPanel, ChromeHeaderPanel, ColorFieldPanel, EventPayloadPanel, FieldRequiredPanel,
    IoParentPanel, ItemListPanel, LayoutShellPanel, NamedSlotsPanel, PipesPanel, SeedBarPanel,
    TemplateOutletPanel, TwoWayPanel,
};
use crate::decor::DemoDecor;

#[component]
pub fn App() -> impl IntoView {
    let tick = RwSignal::new(0_u32);
    let accent = RwSignal::new(String::from("#ce422b"));

    let on_randomize = Callback::new(move |()| {
        tick.update(|n| *n = n.wrapping_add(1));
    });

    view! {
        <DemoDecor tick=tick />
        <main class="demo">
            <header class="demo__hero">
                <div class="demo__hero-copy">
                    <h1 class="demo__title">
                        "rangular demo"
                        <img class="demo__title-logo" src="/logo-256.png" alt="" width="28" height="28" />
                    </h1>
                    <p class="demo__subtitle">
                        "Full fixture corpus from tests/fixtures/, compiled AOT to Leptos CSR / wasm."
                    </p>
                    <SeedBarPanel tick=tick on_randomize=on_randomize />
                </div>
            </header>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"chrome-header"</h2>
                <ChromeHeaderPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"color-field"</h2>
                <ColorFieldPanel tick=tick accent=accent />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"item-list"</h2>
                <ItemListPanel tick=tick />
            </section>

            <section class="demo__panel">
                <div class="demo__panel-head">
                    <h2 class="demo__panel-title">"asset-icon"</h2>
                    <img class="demo__panel-mark" src="/logo-256.png" alt="" width="20" height="20" />
                </div>
                <AssetIconPanel accent=accent />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"layout-shell"</h2>
                <LayoutShellPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"named-slots"</h2>
                <NamedSlotsPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"io-parent + io-child"</h2>
                <IoParentPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"pipes"</h2>
                <PipesPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"two-way"</h2>
                <TwoWayPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"field-required"</h2>
                <FieldRequiredPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"event-payload"</h2>
                <EventPayloadPanel tick=tick />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"template-outlet"</h2>
                <TemplateOutletPanel tick=tick />
            </section>
        </main>
    }
}
