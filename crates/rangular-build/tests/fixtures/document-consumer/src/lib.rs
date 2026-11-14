//! Consumer crate compiled against `rangular-build` document views.

use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/document_view.rs"));
include!(concat!(env!("OUT_DIR"), "/rangular/head_view.rs"));
include!(concat!(env!("OUT_DIR"), "/rangular/body_view.rs"));

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

#[must_use]
pub fn render_page(page: Page) -> String {
    let owner = Owner::new();
    let html = owner.with(|| page_view(page).to_html());
    owner.cleanup();
    html
}

#[cfg(test)]
mod tests {
    use super::{Page, render_page};

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
