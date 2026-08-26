//! Panel intersection + hash sync for the active demo panel.

use std::cell::Cell;
use std::rc::Rc;

use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};

use super::sources::SHOWCASE;
use super::ShowcaseBus;

const SCROLL_DEBOUNCE_MS: u32 = 160;

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

        let bus_obs = bus;
        let debounce_gen = Rc::new(Cell::new(0_u64));
        let debounce_cb = debounce_gen.clone();

        let callback = Closure::wrap(Box::new(move |entries: js_sys::Array, _obs: JsValue| {
            if bus_obs.ignore_observer.get_untracked() {
                return;
            }
            let Some(panel_id) = pick_panel(&entries) else {
                return;
            };
            if bus_obs.active_panel.get_untracked() == panel_id {
                return;
            }

            let gen = debounce_cb.get() + 1;
            debounce_cb.set(gen);
            let bus_wait = bus_obs;
            let debounce_wait = debounce_gen.clone();
            Timeout::new(SCROLL_DEBOUNCE_MS, move || {
                if debounce_wait.get() != gen {
                    return;
                }
                if bus_wait.ignore_observer.get_untracked() {
                    return;
                }
                bus_wait.set_panel(panel_id);
            })
            .forget();
        }) as Box<dyn FnMut(js_sys::Array, JsValue)>);

        let init = IntersectionObserverInit::new();
        init.set_threshold(&js_sys::Array::of3(
            &0.15.into(),
            &0.35.into(),
            &0.55.into(),
        ));
        init.set_root_margin("-8% 0px -40% 0px");

        let Ok(observer) =
            IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &init)
        else {
            return;
        };

        if let Ok(nodes) = document.query_selector_all(".demo__panel") {
            for i in 0..nodes.length() {
                if let Some(node) = nodes.item(i) {
                    if let Ok(el) = node.dyn_into::<Element>() {
                        observer.observe(&el);
                    }
                }
            }
        }

        let bus_hash = bus;
        let on_hash = Closure::wrap(Box::new(move || {
            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    apply_hash(&hash, bus_hash);
                }
            }
        }) as Box<dyn FnMut()>);

        let bus_unlock = bus;
        let on_user_scroll = Closure::wrap(Box::new(move || {
            if bus_unlock.ignore_observer.get_untracked() {
                bus_unlock.ignore_observer.set(false);
            }
        }) as Box<dyn FnMut()>);

        let _ =
            window.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());
        let _ = window
            .add_event_listener_with_callback("wheel", on_user_scroll.as_ref().unchecked_ref());
        let _ = window
            .add_event_listener_with_callback("touchmove", on_user_scroll.as_ref().unchecked_ref());

        callback.forget();
        on_hash.forget();
        on_user_scroll.forget();

        on_cleanup(move || {
            observer.disconnect();
        });
    });
}

fn pick_panel(entries: &js_sys::Array) -> Option<&'static str> {
    let target_y = web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|v| v.as_f64())
        .map_or(160.0, |h| h * 0.28);

    let mut best: Option<(f64, &'static str)> = None;
    for entry in entries.iter() {
        let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() else {
            continue;
        };
        if !entry.is_intersecting() {
            continue;
        }
        let Some(target) = entry.target().dyn_into::<HtmlElement>().ok() else {
            continue;
        };
        let Some(id) = target.get_attribute("id") else {
            continue;
        };
        let Some(panel_id) = intern_panel_id(&id) else {
            continue;
        };
        let rect = target.get_bounding_client_rect();
        let contains = rect.top() <= target_y && rect.bottom() >= target_y;
        let score = if contains {
            (rect.top() - target_y).abs()
        } else {
            10_000.0 + (rect.top() - target_y).abs()
        };
        match best {
            Some((s, _)) if s <= score => {}
            _ => best = Some((score, panel_id)),
        }
    }
    best.map(|(_, id)| id)
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
