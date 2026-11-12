//! Integration fixture for AOT-generated views and the generic Leptos head contract.

use leptos::prelude::*;

include!(concat!(env!("OUT_DIR"), "/generated_head.rs"));

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use leptos::prelude::*;
    use rangular_aot::HostCell;
    use rangular_host::{Host, HostError, Value};
    use serenade_view::{Document, initialize_leptos_ssr};

    use super::generated_head;

    struct HeadHost;

    impl Host for HeadHost {
        fn get(&self, name: &str) -> Option<Value> {
            match name {
                "title" => Some(Value::Str("A & <title>".into())),
                "description" => Some(Value::Str("A & <description>".into())),
                "canonical" => Some(Value::Str("/page?a=1&b=2".into())),
                "json_ld" => Some(Value::Str(r#"{"name":"<product>"}"#.into())),
                _ => None,
            }
        }

        fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
            Ok(Value::Unit)
        }
    }

    #[test]
    fn aot_head_renders_through_the_generic_managed_document_contract() {
        static INITIALIZE: Once = Once::new();
        INITIALIZE.call_once(|| {
            initialize_leptos_ssr().expect("initialize Leptos SSR");
        });

        let rendered = Document::new(
            || generated_head(HostCell::new(HeadHost)),
            || view! { <main>"Body"</main> },
        )
        .render_leptos()
        .expect("render AOT-generated head")
        .into_static_output();
        let head = rendered
            .split_once("<head>")
            .expect("document head")
            .1
            .split_once("</head>")
            .expect("document head end")
            .0;

        assert!(head.contains("<title>A &amp; &lt;title&gt;</title>"));
        assert!(head.contains("name=\"description\""));
        assert!(head.contains("content=\"A &amp; &lt;description&gt;\""));
        assert!(head.contains("href=\"/page?a=1&amp;b=2\""));
        assert_eq!(head.matches("property=\"og:image\"").count(), 2);
        assert!(
            head.contains(
                r#"<script type="application/ld+json">{"name":"\u003cproduct>"}</script>"#
            ),
            "{head}"
        );
        assert!(!head.contains("head-key"));
    }
}
