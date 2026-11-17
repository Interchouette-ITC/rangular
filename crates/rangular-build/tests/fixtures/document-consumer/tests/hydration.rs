//! Hydrates the native SSR output of the generated counter view in a browser.
//!
//! `RANGULAR_SSR_HTML` must name the file written by
//! `cargo run --example render_counter` (see `make test-hydration`). The
//! browser build uses the same template and `Counter::initial()` state.

#![cfg(all(target_arch = "wasm32", feature = "hydrate"))]

use leptos::mount::{hydrate_from, mount_to};
use rangular_document_consumer_fixture::counter_app;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{Document, Element, HtmlElement, Node};

wasm_bindgen_test_configure!(run_in_browser);

const SERVER_HTML: &str = include_str!(env!("RANGULAR_SSR_HTML"));

fn document() -> Document {
    web_sys::window()
        .expect("browser window")
        .document()
        .expect("browser document")
}

fn server_rendered_root() -> HtmlElement {
    let document = document();
    let root = document
        .create_element("div")
        .expect("create root")
        .dyn_into::<HtmlElement>()
        .expect("root element");
    root.set_inner_html(SERVER_HTML);
    document
        .body()
        .expect("document body")
        .append_child(&root)
        .expect("append root");
    root
}

fn select(root: &Element, selector: &str) -> Element {
    root.query_selector(selector)
        .expect("valid selector")
        .unwrap_or_else(|| panic!("missing {selector}"))
}

fn text(root: &Element, selector: &str) -> String {
    select(root, selector).text_content().unwrap_or_default()
}

/// DOM nodes produced by the server that hydration must adopt, not replace.
struct ServerNodes {
    section: Element,
    title: Element,
    value: Element,
    value_text: Node,
    button: Element,
    list: Element,
    items: Vec<Element>,
}

impl ServerNodes {
    fn capture(root: &Element) -> Self {
        let value = select(root, "#counter-value");
        let list = select(root, "#counter-items");
        let children = list.children();
        let items = (0..children.length())
            .filter_map(|index| children.item(index))
            .collect();
        Self {
            section: select(root, "section.counter"),
            title: select(root, "#counter-title"),
            value_text: value.last_child().expect("count text node"),
            value,
            button: select(root, "#counter-increment"),
            list,
            items,
        }
    }

    fn retained_in(&self, root: &Element) -> Result<(), String> {
        if root.child_element_count() != 1 {
            return Err(format!(
                "expected one rendered section, found {}",
                root.child_element_count()
            ));
        }
        let current = Self::capture(root);
        let pairs: [(&str, &Node, &Node); 6] = [
            ("section", &self.section, &current.section),
            ("title", &self.title, &current.title),
            ("count paragraph", &self.value, &current.value),
            ("count text", &self.value_text, &current.value_text),
            ("button", &self.button, &current.button),
            ("list", &self.list, &current.list),
        ];
        for (label, server, now) in pairs {
            if !server.is_same_node(Some(now)) {
                return Err(format!("server {label} node was replaced"));
            }
        }
        for (index, item) in self.items.iter().enumerate() {
            let same = current
                .items
                .get(index)
                .is_some_and(|now| item.is_same_node(Some(now)));
            if !same {
                return Err(format!("server list item {index} was replaced"));
            }
        }
        Ok(())
    }
}

async fn next_task() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .expect("browser window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .expect("schedule task");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("wait for task");
}

#[wasm_bindgen_test]
async fn hydration_adopts_server_html_and_becomes_interactive() {
    assert!(SERVER_HTML.contains("<!>"), "expected SSR markers");
    let root = server_rendered_root();
    let server = ServerNodes::capture(&root);
    assert_eq!(server.items.len(), 2);
    assert_eq!(text(&root, "#counter-value"), "Count:0");

    let handle = hydrate_from(root.clone(), counter_app);
    next_task().await;
    server
        .retained_in(&root)
        .expect("hydration kept server DOM");
    assert_eq!(text(&root, "#counter-state"), "zero");

    root.query_selector("#counter-increment")
        .expect("valid selector")
        .expect("button")
        .dyn_into::<HtmlElement>()
        .expect("button element")
        .click();
    next_task().await;

    assert_eq!(text(&root, "#counter-value"), "Count:1");
    assert_eq!(server.value_text.text_content().as_deref(), Some("1"));
    assert_eq!(text(&root, "#counter-state"), "positive");
    let items = select(&root, "#counter-items").children();
    assert_eq!(items.length(), 3);
    assert_eq!(
        items
            .item(2)
            .and_then(|item| item.text_content())
            .as_deref(),
        Some("2:item-1")
    );
    server
        .retained_in(&root)
        .expect("updates reused server DOM");

    drop(handle);
    root.remove();
}

/// Guards the test itself: a fresh client render must not pass the check.
#[wasm_bindgen_test]
async fn fresh_client_render_is_detected() {
    let root = server_rendered_root();
    let server = ServerNodes::capture(&root);

    root.set_inner_html("");
    let handle = mount_to(root.clone(), counter_app);
    next_task().await;

    assert!(text(&root, "#counter-value").ends_with('0'));
    let error = server
        .retained_in(&root)
        .expect_err("client remount must not look hydrated");
    assert!(error.contains("replaced"), "{error}");

    drop(handle);
    root.remove();
}
