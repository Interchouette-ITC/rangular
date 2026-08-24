use leptos::prelude::*;

use crate::components::{
    AssetIconPanel, ChromeHeaderPanel, ColorFieldPanel, EventPayloadPanel, FieldRequiredPanel,
    IoChildDemoPanel, ItemListPanel, LayoutShellPanel, NamedSlotsPanel, PipesPanel, SeedBarPanel,
    TemplateOutletPanel, TwoWayPanel,
};
use crate::decor::DemoDecor;

const FIXTURE_NAV: [(&str, &str); 12] = [
    ("chrome-header", "chrome-header"),
    ("color-field", "color-field"),
    ("asset-icon", "asset-icon"),
    ("layout-shell", "layout-shell"),
    ("named-slots", "named-slots"),
    ("io-child", "io-child"),
    ("pipes", "pipes"),
    ("two-way", "two-way"),
    ("field-required", "field-required"),
    ("event-payload", "event-payload"),
    ("template-outlet", "template-outlet"),
    ("item-list", "item-list"),
];

#[component]
pub fn App() -> impl IntoView {
    let applied_seed = RwSignal::new(String::new());
    let accent = RwSignal::new(String::from("#ce422b"));

    view! {
        <DemoDecor applied_seed=applied_seed />
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
                    <SeedBarPanel applied_seed=applied_seed />
                    <nav class="demo__nav" aria-label="Fixture panels">
                        {FIXTURE_NAV
                            .iter()
                            .map(|(id, label)| {
                                view! {
                                    <a class="demo__nav-link" href=format!("#{id}")>{*label}</a>
                                }
                            })
                            .collect_view()}
                    </nav>
                </div>
            </header>

            <section class="demo__panel" id="chrome-header">
                <h2 class="demo__panel-title">"chrome-header"</h2>
                <ChromeHeaderPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="color-field">
                <h2 class="demo__panel-title">"color-field"</h2>
                <ColorFieldPanel applied_seed=applied_seed accent=accent />
            </section>

            <section class="demo__panel" id="asset-icon">
                <div class="demo__panel-head">
                    <h2 class="demo__panel-title">"asset-icon"</h2>
                    <img class="demo__panel-mark" src="/logo-256.png" alt="" width="20" height="20" />
                </div>
                <AssetIconPanel accent=accent />
            </section>

            <section class="demo__panel" id="layout-shell">
                <h2 class="demo__panel-title">"layout-shell"</h2>
                <LayoutShellPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="named-slots">
                <h2 class="demo__panel-title">"named-slots"</h2>
                <NamedSlotsPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel demo__panel--io-child" id="io-child">
                <h2 class="demo__panel-title">"io-child"</h2>
                <IoChildDemoPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="pipes">
                <h2 class="demo__panel-title">"pipes"</h2>
                <PipesPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="two-way">
                <h2 class="demo__panel-title">"two-way"</h2>
                <TwoWayPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="field-required">
                <h2 class="demo__panel-title">"field-required"</h2>
                <FieldRequiredPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="event-payload">
                <h2 class="demo__panel-title">"event-payload"</h2>
                <EventPayloadPanel />
            </section>

            <section class="demo__panel" id="template-outlet">
                <h2 class="demo__panel-title">"template-outlet"</h2>
                <TemplateOutletPanel applied_seed=applied_seed />
            </section>

            <section class="demo__panel" id="item-list">
                <h2 class="demo__panel-title">"item-list"</h2>
                <ItemListPanel applied_seed=applied_seed />
            </section>
        </main>
    }
}
