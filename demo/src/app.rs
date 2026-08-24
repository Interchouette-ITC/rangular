use leptos::prelude::*;

use crate::components::{ItemListPanel, SeedBarPanel};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="demo">
            <header class="demo__hero">
                <img class="demo__logo" src="/logo-256.png" alt="rangular logo" width="64" height="64" />
                <div>
                    <h1 class="demo__title">"rangular demo"</h1>
                    <p class="demo__subtitle">
                        "Fixture templates compiled AOT to Leptos CSR / wasm."
                    </p>
                </div>
            </header>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"seed-bar fixture"</h2>
                <SeedBarPanel />
            </section>

            <section class="demo__panel">
                <h2 class="demo__panel-title">"item-list fixture"</h2>
                <ItemListPanel />
            </section>
        </main>
    }
}
