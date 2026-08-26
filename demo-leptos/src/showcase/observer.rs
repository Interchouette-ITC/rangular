//! Hover and hash sync for the active demo panel (no scroll spy).

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Event, Window};

use super::sources::SHOWCASE;
use super::ShowcaseBus;

pub fn watch_panels(bus: ShowcaseBus) {
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };

        if let Ok(hash) = window.location().hash() {
            apply_hash(&hash, bus);
        }

        bind_hash_change(&window, bus);
        bind_panel_hover(&document, bus);
    });
}

fn bind_hash_change(window: &Window, bus: ShowcaseBus) {
    let on_hash = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Ok(hash) = window.location().hash() {
                apply_hash(&hash, bus);
            }
        }
    }) as Box<dyn FnMut()>);

    let _ = window.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());
    on_hash.forget();
}

fn bind_panel_hover(document: &Document, bus: ShowcaseBus) {
    let on_panel_enter = Closure::wrap(Box::new(move |event: Event| {
        let Some(target) = event
            .current_target()
            .and_then(|t| t.dyn_into::<Element>().ok())
        else {
            return;
        };
        let Some(id) = target.get_attribute("id") else {
            return;
        };
        if let Some(panel_id) = intern_panel_id(&id) {
            bus.set_panel_from_hover(panel_id);
        }
    }) as Box<dyn FnMut(Event)>);

    if let Ok(nodes) = document.query_selector_all(".demo__panel[id]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    let _ = el.add_event_listener_with_callback(
                        "pointerenter",
                        on_panel_enter.as_ref().unchecked_ref(),
                    );
                }
            }
        }
    }

    on_panel_enter.forget();
}

fn apply_hash(hash: &str, bus: ShowcaseBus) {
    let id = hash.trim_start_matches('#');
    if id.is_empty() {
        return;
    }
    if let Some(panel_id) = intern_panel_id(id) {
        bus.set_panel_from_nav(panel_id);
    }
}

fn intern_panel_id(id: &str) -> Option<&'static str> {
    SHOWCASE
        .iter()
        .map(|e| e.panel_id)
        .find(|&panel| panel == id)
}
