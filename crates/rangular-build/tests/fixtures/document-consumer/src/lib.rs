//! Consumer crate compiled against `rangular-build` views.
//!
//! It depends on `rangular` (facade) and `rangular-build` only; generated code
//! must therefore resolve every Rangular path through the facade.

use leptos::prelude::*;
use rangular::aot::HostCell;
use rangular::host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/document_view.rs"));
include!(concat!(env!("OUT_DIR"), "/rangular/head_view.rs"));
include!(concat!(env!("OUT_DIR"), "/rangular/body_view.rs"));
include!(concat!(env!("OUT_DIR"), "/rangular/counter_view.rs"));

/// Generated Rust for the counter component, for facade-path assertions.
pub const COUNTER_VIEW_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/rangular/counter_view.rs"));

#[derive(Clone)]
pub struct Page {
    pub title: String,
    pub description: String,
    pub canonical: String,
}

impl Host for Page {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "title" => Some(Value::Str(self.title.clone())),
            "description" => Some(Value::Str(self.description.clone())),
            "canonical" => Some(Value::Str(self.canonical.clone())),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

pub fn page_view(page: Page) -> impl IntoView {
    let (head_page, body_page) = (page.clone(), page.clone());
    document_view(
        HostCell::new(page),
        Box::new(move || head_view(HostCell::new(head_page)).into_any()),
        Box::new(move || body_view(HostCell::new(body_page)).into_any()),
    )
}

/// Interactive state shared by the server render and the browser hydration.
#[derive(Clone, Copy)]
pub struct Counter {
    count: RwSignal<u32>,
    items: RwSignal<Vec<String>>,
}

impl Counter {
    /// Initial state used identically by the SSR and hydrate builds.
    #[must_use]
    pub fn initial() -> Self {
        Self {
            count: RwSignal::new(0),
            items: RwSignal::new(vec!["alpha".into(), "beta".into()]),
        }
    }
}

impl Host for Counter {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "title" => Some(Value::Str("Hydrated counter".into())),
            "count" => Some(Value::Num(f64::from(self.count.get()))),
            "positive" => Some(Value::Bool(self.count.get() > 0)),
            "items" => Some(Value::List(
                self.items.get().into_iter().map(Value::Str).collect(),
            )),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "increment" {
            self.count.update(|count| *count += 1);
            let label = format!("item-{}", self.count.get_untracked());
            self.items.update(|items| items.push(label));
        }
        Ok(Value::Unit)
    }
}

/// The counter application rendered on the server and hydrated in the browser.
pub fn counter_app() -> impl IntoView {
    counter_view(HostCell::new(Counter::initial()))
}

/// Renders the counter application to HTML exactly as served to browsers.
#[must_use]
pub fn render_counter() -> String {
    let owner = Owner::new();
    let html = owner.with(|| counter_app().to_html());
    owner.cleanup();
    html
}

#[must_use]
pub fn render_page(page: Page) -> String {
    let owner = Owner::new();
    let html = owner.with(|| page_view(page).to_html());
    owner.cleanup();
    html
}

#[cfg(test)]
mod tests {
    use super::{COUNTER_VIEW_SOURCE, Page, render_counter, render_page};

    #[test]
    fn generated_views_reference_rangular_only_through_the_facade() {
        for path in [
            "rangular_aot",
            "rangular_css",
            "rangular_expr",
            "rangular_host",
            "rangular_macros",
            "rangular_parser",
            "rangular_runtime",
        ] {
            let direct = COUNTER_VIEW_SOURCE
                .match_indices(path)
                .any(|(at, _)| !COUNTER_VIEW_SOURCE[..at].ends_with('_'));
            assert!(!direct, "generated code references `{path}` directly");
        }
        assert!(COUNTER_VIEW_SOURCE.contains("rangular::host::Host"));
        assert!(COUNTER_VIEW_SOURCE.contains("rangular::aot::HostCell"));
    }

    #[test]
    fn counter_renders_on_the_server_with_hydration_markers() {
        assert_eq!(
            render_counter(),
            concat!(
                "<section aria-label=\"Hydrated counter\" class=\"counter\">",
                "<h2 id=\"counter-title\">Hydrated counter</h2>",
                "<p id=\"counter-value\">Count:<!>0</p>",
                "<p id=\"counter-state\">zero</p>",
                "<button id=\"counter-increment\" type=\"button\">Increment</button>",
                "<ul id=\"counter-items\"><li>0<!>:<!>alpha</li><li>1<!>:<!>beta</li><!></ul>",
                "</section>",
            )
        );
    }

    #[test]
    fn generated_document_views_render_a_complete_document() {
        let html = render_page(Page {
            title: "Docs & <Guides>".into(),
            description: "Read \"this\" & that".into(),
            canonical: "/docs?a=1&b=2".into(),
        });

        assert_eq!(
            html,
            concat!(
                "<!DOCTYPE html><html lang=\"fr\"><head><meta charset=\"utf-8\">",
                "<title>Docs &amp; &lt;Guides&gt;</title>",
                "<meta name=\"description\" content=\"Read &quot;this&quot; &amp; that\">",
                "<link rel=\"canonical\" href=\"/docs?a=1&amp;b=2\"></head>",
                "<body><main><h1>Docs &amp; &lt;Guides&gt;</h1></main></body></html>",
            )
        );
    }
}
