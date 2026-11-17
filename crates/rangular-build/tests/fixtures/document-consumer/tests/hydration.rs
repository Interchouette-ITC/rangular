//! Hydrates the native SSR output of the generated counter view in a browser.
//!
//! `RANGULAR_SSR_HTML` must name the file written by
//! `cargo run --example render_counter` (see `make test-hydration`). The
//! browser build uses the same template and `Counter::initial()` state.
//!
//! Leptos does not check tag names or every marker while hydrating, so the
//! server HTML is first compared node by node with a client render of the same
//! view in this build; any structural mismatch fails the test.

#![cfg(all(target_arch = "wasm32", feature = "hydrate"))]

use leptos::mount::{hydrate_from, mount_to};
use rangular_document_consumer_fixture::counter_app;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{Document, Element, ErrorEvent, ErrorEventInit, HtmlElement, Node};

wasm_bindgen_test_configure!(run_in_browser);

const SERVER_HTML: &str = include_str!(env!("RANGULAR_SSR_HTML"));

// Leptos reports hydration mismatches with `console.error` before panicking,
// and a panic inside an event handler or effect only surfaces as an uncaught
// browser error. Record all of them so the test fails on any diagnostic.
#[wasm_bindgen(inline_js = r#"
export function start_diagnostics() {
  const seen = [];
  const describe = (value) =>
    value instanceof Node ? (value.outerHTML ?? value.textContent) : String(value);
  const originals = {};
  for (const level of ["error", "warn"]) {
    originals[level] = console[level];
    console[level] = (...args) => {
      seen.push(`console.${level}: ${args.map(describe).join(" ")}`);
      originals[level].apply(console, args);
    };
  }
  const onError = (event) =>
    seen.push(`uncaught error: ${event.message || describe(event.error)}`);
  const onRejection = (event) =>
    seen.push(`unhandled rejection: ${describe(event.reason)}`);
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);
  return {
    stop() {
      for (const level of Object.keys(originals)) console[level] = originals[level];
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
      return seen.slice();
    },
  };
}
"#)]
extern "C" {
    type DiagnosticsCapture;

    fn start_diagnostics() -> DiagnosticsCapture;

    #[wasm_bindgen(method)]
    fn stop(this: &DiagnosticsCapture) -> js_sys::Array;
}

/// Browser diagnostics recorded between `start` and `finish`.
struct Diagnostics(DiagnosticsCapture);

impl Diagnostics {
    fn start() -> Self {
        Self(start_diagnostics())
    }

    fn finish(self) -> Vec<String> {
        self.0
            .stop()
            .iter()
            .map(|message| message.as_string().unwrap_or_default())
            .collect()
    }
}

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

/// Parses `html` into a detached container without touching the page.
fn parse(html: &str) -> Element {
    let container = document().create_element("div").expect("create container");
    container.set_inner_html(html);
    container
}

fn describe_node(node: &Node) -> String {
    match node.dyn_ref::<Element>() {
        Some(element) => {
            let mut attributes: Vec<String> = element
                .get_attribute_names()
                .iter()
                .filter_map(|name| name.as_string())
                .map(|name| {
                    let value = element.get_attribute(&name).unwrap_or_default();
                    format!("{name}={value:?}")
                })
                .collect();
            attributes.sort();
            format!("<{} {}>", node.node_name().to_lowercase(), attributes.join(" "))
        }
        None => format!(
            "{}({:?})",
            node.node_name(),
            node.node_value().unwrap_or_default()
        ),
    }
}

fn is_text(node: Option<&Node>) -> bool {
    node.is_some_and(|node| node.node_type() == Node::TEXT_NODE)
}

/// Child nodes, minus the empty `<!>` comments SSR writes between adjacent
/// text nodes only so the HTML parser keeps them apart. Removing such a
/// separator still shows up: the parser then merges the two texts into one.
fn children(node: &Node) -> Vec<Node> {
    let mut all = Vec::new();
    let mut next = node.first_child();
    while let Some(child) = next {
        next = child.next_sibling();
        all.push(child);
    }
    (0..all.len())
        .filter(|&index| {
            let node = &all[index];
            let separator = node.node_type() == Node::COMMENT_NODE
                && node.node_value().unwrap_or_default().is_empty()
                && index > 0
                && is_text(all.get(index - 1))
                && is_text(all.get(index + 1));
            !separator
        })
        .map(|index| all[index].clone())
        .collect()
}

/// First difference between two parsed trees: node kind, tag, attributes,
/// text, and hydration marker comments must all match.
fn first_difference(expected: &Node, actual: &Node, path: &str) -> Option<String> {
    let (want, have) = (describe_node(expected), describe_node(actual));
    if want != have {
        return Some(format!("{path}: expected {want}, found {have}"));
    }
    let (want, have) = (children(expected), children(actual));
    for (index, (want, have)) in want.iter().zip(&have).enumerate() {
        let child_path = format!("{path}/{index}:{}", want.node_name().to_lowercase());
        if let Some(difference) = first_difference(want, have, &child_path) {
            return Some(difference);
        }
    }
    (want.len() != have.len()).then(|| {
        format!(
            "{path}: expected {} child nodes, found {}",
            want.len(),
            have.len()
        )
    })
}

/// Fails unless `server_html` has exactly the structure the hydrating view
/// expects: the DOM a client render of the same view creates in this build.
fn hydration_mismatch(server_html: &str) -> Option<String> {
    let expected = document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<HtmlElement>()
        .expect("container element");
    let handle = mount_to(expected.clone(), counter_app);
    let actual = parse(server_html);
    let difference = first_difference(&expected, &actual, "root");
    drop(handle);
    difference
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
    if let Some(difference) = hydration_mismatch(SERVER_HTML) {
        panic!("server HTML does not match the hydrating view: {difference}");
    }
    let root = server_rendered_root();
    let server = ServerNodes::capture(&root);
    assert_eq!(server.items.len(), 2);
    assert_eq!(text(&root, "#counter-value"), "Count:0");

    let diagnostics = Diagnostics::start();
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

    let messages = diagnostics.finish();
    assert!(
        messages.is_empty(),
        "hydration reported browser diagnostics:\n{}",
        messages.join("\n")
    );

    drop(handle);
    root.remove();
}

/// Guards the diagnostics capture itself: each reported channel must be seen.
#[wasm_bindgen_test]
fn diagnostics_capture_reports_console_and_uncaught_errors() {
    let diagnostics = Diagnostics::start();
    web_sys::console::error_1(&"simulated hydration mismatch".into());
    web_sys::console::warn_1(&"simulated warning".into());
    let init = ErrorEventInit::new();
    init.set_message("simulated uncaught panic");
    let event = ErrorEvent::new_with_event_init_dict("error", &init).expect("error event");
    web_sys::window()
        .expect("browser window")
        .dispatch_event(&event)
        .expect("dispatch error event");

    assert_eq!(
        diagnostics.finish(),
        [
            "console.error: simulated hydration mismatch",
            "console.warn: simulated warning",
            "uncaught error: simulated uncaught panic",
        ]
    );
}

/// Guards the structural check: mismatches Leptos hydrates silently must fail.
#[wasm_bindgen_test]
fn structural_mismatches_are_detected() {
    assert_eq!(hydration_mismatch(SERVER_HTML), None);
    let corruptions = [
        (
            "tag renamed",
            SERVER_HTML
                .replacen("<h2 ", "<h3 ", 1)
                .replacen("</h2>", "</h3>", 1),
            "expected <h2",
        ),
        (
            "marker removed",
            SERVER_HTML.replacen("Count:<!>0", "Count:0", 1),
            "expected #text(\"Count:\"), found #text(\"Count:0\")",
        ),
        (
            "list marker removed",
            SERVER_HTML.replacen("</li><!></ul>", "</li></ul>", 1),
            "expected 3 child nodes, found 2",
        ),
        (
            "attribute changed",
            SERVER_HTML.replacen("id=\"counter-state\"", "id=\"counter-status\"", 1),
            "id=\"counter-state\"",
        ),
        (
            "text changed",
            SERVER_HTML.replacen(">zero<", ">none<", 1),
            "expected #text(\"zero\")",
        ),
        (
            "element added",
            SERVER_HTML.replacen("</ul>", "<li>extra</li></ul>", 1),
            "expected 3 child nodes, found 4",
        ),
    ];
    for (label, html, expected) in corruptions {
        assert_ne!(html, SERVER_HTML, "{label}: corruption did not apply");
        let difference = hydration_mismatch(&html)
            .unwrap_or_else(|| panic!("{label}: mismatch was not detected"));
        assert!(difference.contains(expected), "{label}: {difference}");
    }
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
