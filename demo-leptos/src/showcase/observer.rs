//! Panel scroll / hover / hash sync for the active demo panel.

use std::cell::Cell;
use std::rc::Rc;

use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Event, HtmlElement, Window};

use super::sources::SHOWCASE;
use super::ShowcaseBus;

const SCROLL_DEBOUNCE_MS: u32 = 80;
/// Viewport fraction used as the scroll-spy focus line (from the top).
const FOCUS_Y_RATIO: f64 = 0.3;

type ScrollPick = Rc<dyn Fn()>;

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

        let debounce_gen = Rc::new(Cell::new(0_u64));
        let schedule_scroll_pick = make_scroll_pick(bus, document.clone(), debounce_gen.clone());

        Timeout::new(0, {
            let schedule_scroll_pick = Rc::clone(&schedule_scroll_pick);
            move || schedule_scroll_pick()
        })
        .forget();

        bind_window_listeners(&window, bus, &schedule_scroll_pick);
        bind_panel_hover(&document, bus, debounce_gen, schedule_scroll_pick);
    });
}

fn make_scroll_pick(
    bus: ShowcaseBus,
    document: Document,
    debounce_gen: Rc<Cell<u64>>,
) -> ScrollPick {
    Rc::new(move || {
        if bus.ignore_observer.get_untracked() || bus.hover_lock.get_untracked() {
            return;
        }
        let gen = debounce_gen.get() + 1;
        debounce_gen.set(gen);
        let debounce_wait = debounce_gen.clone();
        let document = document.clone();
        Timeout::new(SCROLL_DEBOUNCE_MS, move || {
            if debounce_wait.get() != gen {
                return;
            }
            if bus.ignore_observer.get_untracked() || bus.hover_lock.get_untracked() {
                return;
            }
            if let Some(panel_id) = pick_panel_from_dom(&document) {
                bus.set_panel(panel_id);
            }
        })
        .forget();
    })
}

fn bind_window_listeners(window: &Window, bus: ShowcaseBus, schedule_scroll_pick: &ScrollPick) {
    let on_scroll = {
        let schedule_scroll_pick = Rc::clone(schedule_scroll_pick);
        Closure::wrap(Box::new(move || {
            if bus.ignore_observer.get_untracked() {
                bus.ignore_observer.set(false);
            }
            schedule_scroll_pick();
        }) as Box<dyn FnMut()>)
    };

    let on_resize = {
        let schedule_scroll_pick = Rc::clone(schedule_scroll_pick);
        Closure::wrap(Box::new(move || {
            schedule_scroll_pick();
        }) as Box<dyn FnMut()>)
    };

    let on_hash = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Ok(hash) = window.location().hash() {
                apply_hash(&hash, bus);
            }
        }
    }) as Box<dyn FnMut()>);

    let _ = window.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
    let _ = window.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref());
    let _ = window.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());

    on_scroll.forget();
    on_resize.forget();
    on_hash.forget();
}

fn bind_panel_hover(
    document: &Document,
    bus: ShowcaseBus,
    debounce_gen: Rc<Cell<u64>>,
    schedule_scroll_pick: ScrollPick,
) {
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
            debounce_gen.set(debounce_gen.get() + 1);
            bus.ignore_observer.set(false);
            bus.set_panel_from_hover(panel_id);
        }
    }) as Box<dyn FnMut(Event)>);

    let on_panel_leave = Closure::wrap(Box::new(move |_event: Event| {
        bus.hover_lock.set(false);
        schedule_scroll_pick();
    }) as Box<dyn FnMut(Event)>);

    if let Ok(nodes) = document.query_selector_all(".demo__panel[id]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    let _ = el.add_event_listener_with_callback(
                        "pointerenter",
                        on_panel_enter.as_ref().unchecked_ref(),
                    );
                    let _ = el.add_event_listener_with_callback(
                        "pointerleave",
                        on_panel_leave.as_ref().unchecked_ref(),
                    );
                }
            }
        }
    }

    on_panel_enter.forget();
    on_panel_leave.forget();
}

/// Active panel: the one containing the focus line, else the last whose top
/// has crossed it. At the document bottom, always prefer the last panel.
fn pick_panel_from_dom(document: &Document) -> Option<&'static str> {
    let window = web_sys::window()?;
    let focus_y = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .map_or(200.0, |h| h * FOCUS_Y_RATIO);

    let Ok(nodes) = document.query_selector_all(".demo__panel[id]") else {
        return None;
    };

    let mut ordered: Vec<(&'static str, f64, f64)> = Vec::new();
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else {
            continue;
        };
        let Ok(el) = node.dyn_into::<HtmlElement>() else {
            continue;
        };
        let Some(id) = el.get_attribute("id") else {
            continue;
        };
        let Some(panel_id) = intern_panel_id(&id) else {
            continue;
        };
        let rect = el.get_bounding_client_rect();
        ordered.push((panel_id, rect.top(), rect.bottom()));
    }
    if ordered.is_empty() {
        return None;
    }

    let scroll_y = window.scroll_y().unwrap_or(0.0);
    let viewport_h = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let doc_h = document
        .document_element()
        .map_or(0.0, |el| f64::from(el.scroll_height()));
    let near_bottom = scroll_y + viewport_h >= doc_h - 8.0;

    // Top of page: keep the first panel until the user scrolls past the hero.
    if scroll_y < 32.0 {
        return ordered.first().map(|(id, _, _)| *id);
    }

    // At the page end, prefer the lowest panel that is still on screen.
    if near_bottom {
        for (panel_id, top, bottom) in ordered.iter().rev() {
            if *top < viewport_h * 0.85 && *bottom > 0.0 {
                return Some(*panel_id);
            }
        }
    }

    let mut crossed: Option<&'static str> = None;
    for (panel_id, top, bottom) in &ordered {
        if *top <= focus_y && *bottom > focus_y {
            return Some(*panel_id);
        }
        if *top <= focus_y {
            crossed = Some(*panel_id);
        }
    }

    crossed.or_else(|| ordered.first().map(|(id, _, _)| *id))
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
