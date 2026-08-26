//! Shared showcase signals: active panel, expand state, and binding pulses.

use gloo_timers::callback::Timeout;
use leptos::prelude::*;

const STORAGE_KEY: &str = "rangular-demo-code-rail-open";
const PULSE_MS: u32 = 700;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShowcasePulse {
    #[default]
    None,
    MuteToggle,
}

#[derive(Clone, Copy)]
pub struct ShowcaseBus {
    pub active_panel: RwSignal<&'static str>,
    pub expanded: RwSignal<bool>,
    pub file_index: RwSignal<usize>,
    pub pulse: RwSignal<ShowcasePulse>,
    pub status: RwSignal<&'static str>,
}

impl ShowcaseBus {
    pub fn new() -> Self {
        let expanded = RwSignal::new(default_expanded());
        Self {
            active_panel: RwSignal::new("chrome-header"),
            expanded,
            file_index: RwSignal::new(0),
            pulse: RwSignal::new(ShowcasePulse::None),
            status: RwSignal::new("hover a panel"),
        }
    }

    pub fn set_panel(&self, panel_id: &'static str) {
        if self.active_panel.get_untracked() == panel_id {
            return;
        }
        self.active_panel.set(panel_id);
        self.file_index
            .set(crate::showcase::sources::default_file_index(panel_id));
        self.pulse.set(ShowcasePulse::None);
    }

    /// Pointer enter over a fixture panel; stays until another panel or hash nav.
    pub fn set_panel_from_hover(&self, panel_id: &'static str) {
        self.set_panel(panel_id);
        self.status.set("hover");
    }

    /// Pin a panel from hash / in-page nav.
    pub fn set_panel_from_nav(&self, panel_id: &'static str) {
        self.set_panel(panel_id);
        self.status.set("hash nav");
    }

    pub fn persist_expanded(&self) {
        let value = if self.expanded.get_untracked() {
            "expanded"
        } else {
            "collapsed"
        };
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(STORAGE_KEY, value);
            }
        }
    }

    pub fn pulse_mute(&self) {
        self.pulse_io_child(ShowcasePulse::MuteToggle, "event emitted · muteToggle");
    }

    fn pulse_io_child(&self, pulse: ShowcasePulse, status: &'static str) {
        if self.active_panel.get_untracked() != "io-parent" {
            return;
        }
        if self.file_index.get_untracked() != 3 {
            self.file_index.set(3);
        }
        self.pulse.set(pulse);
        self.status.set(status);
        self.clear_pulse_later();
    }

    fn clear_pulse_later(&self) {
        let pulse = self.pulse;
        let status = self.status;
        Timeout::new(PULSE_MS, move || {
            pulse.set(ShowcasePulse::None);
            status.set("hover a panel");
        })
        .forget();
    }
}

fn default_expanded() -> bool {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item(STORAGE_KEY) {
                return value == "expanded";
            }
        }
        if let Ok(width) = window.inner_width() {
            if let Some(w) = width.as_f64() {
                // First visit on narrow viewports: start collapsed so the demo stays usable.
                if w <= 1024.0 {
                    return false;
                }
            }
        }
    }
    true
}
