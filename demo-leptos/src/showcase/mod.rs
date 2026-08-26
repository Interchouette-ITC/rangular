//! Floating code showcase rail (`kode-leptos` presentation surface).

mod bus;
mod observer;
mod sources;

pub use bus::{ShowcaseBus, ShowcasePulse};

use std::sync::Arc;

use gloo_timers::callback::Timeout;
use kode_leptos::{CodeEditor, EditorHandle, Marker, MarkerSeverity, Position, Theme};
use leptos::prelude::*;

use self::sources::{
    file_extension, file_stem, group_label_for_stem, is_io_child_html, source_for, truncate_middle,
    ShowcaseFile, ShowcaseLang, TAB_STEM_MAX, TRINITY_LEN,
};

const CONTENT_FADE_MS: u32 = 120;

#[component]
pub fn CodeShowcase() -> impl IntoView {
    let bus = expect_context::<ShowcaseBus>();
    let initial = source_for("chrome-header", 0);
    let content = RwSignal::new(
        initial
            .map(|(_, f)| f.source.to_owned())
            .unwrap_or_default(),
    );
    let active_lang = RwSignal::new(initial.map_or(ShowcaseLang::Html, |(_, f)| f.lang));
    let handle = RwSignal::new(None::<EditorHandle>);
    let tab_files = RwSignal::new(initial.map(|(files, _)| files));
    let editor_fading = RwSignal::new(false);

    sync_content(bus, content, tab_files, active_lang, editor_fading);
    sync_markers(bus, handle);
    observer::watch_panels(bus);

    let on_ready = Arc::new(move |h: EditorHandle| {
        handle.set(Some(h));
    });

    view! {
        <Backdrop bus=bus />
        <aside
            class=move || showcase_class(bus.expanded.get())
            aria-label="Code showcase"
        >
            <ToggleButton bus=bus />
            <div id="showcase-panel" class="showcase__panel">
                <Chrome bus=bus tab_files=tab_files active_lang=active_lang />
                <EditorPane
                    content=content
                    active_lang=active_lang
                    editor_fading=editor_fading
                    on_ready=on_ready
                />
                <StatusBar bus=bus />
            </div>
        </aside>
    }
}

#[component]
fn Backdrop(bus: ShowcaseBus) -> impl IntoView {
    view! {
        <button
            type="button"
            class=move || {
                if bus.expanded.get() {
                    "showcase__backdrop showcase__backdrop--visible"
                } else {
                    "showcase__backdrop"
                }
            }
            aria-label="Close code panel"
            tabindex=move || if bus.expanded.get() { "0" } else { "-1" }
            on:click=move |_| {
                bus.expanded.set(false);
                bus.persist_expanded();
            }
        ></button>
    }
}

fn sync_content(
    bus: ShowcaseBus,
    content: RwSignal<String>,
    tab_files: RwSignal<Option<&'static [ShowcaseFile]>>,
    active_lang: RwSignal<ShowcaseLang>,
    editor_fading: RwSignal<bool>,
) {
    Effect::new(move |_| {
        let panel = bus.active_panel.get();
        let idx = bus.file_index.get();

        let apply = move || {
            let Some((files, file)) = source_for(panel, idx) else {
                content.set(String::new());
                tab_files.set(None);
                return;
            };
            tab_files.set(Some(files));
            content.set(file.source.to_owned());
            active_lang.set(file.lang);
        };

        if content.get_untracked().is_empty() {
            apply();
            return;
        }

        editor_fading.set(true);
        Timeout::new(CONTENT_FADE_MS, move || {
            apply();
            editor_fading.set(false);
        })
        .forget();
    });
}

fn sync_markers(bus: ShowcaseBus, handle: RwSignal<Option<EditorHandle>>) {
    Effect::new(move |_| {
        let pulse = bus.pulse.get();
        let panel = bus.active_panel.get();
        let idx = bus.file_index.get();
        let Some(h) = handle.get() else {
            return;
        };
        if !is_io_child_html(panel, idx) {
            h.clear_markers();
            return;
        }
        h.set_markers(markers_for(pulse));
    });
}

fn markers_for(pulse: ShowcasePulse) -> Vec<Marker> {
    match pulse {
        ShowcasePulse::None => {
            vec![line_marker(7, "muteToggle binding", MarkerSeverity::Hint)]
        }
        ShowcasePulse::MuteToggle => {
            vec![line_marker(
                7,
                "(click)=\"muteToggle($event)\"",
                MarkerSeverity::Info,
            )]
        }
    }
}

fn line_marker(line: usize, message: &str, severity: MarkerSeverity) -> Marker {
    Marker {
        start: Position::new(line, 0),
        end: Position::new(line, 120),
        message: message.to_owned(),
        severity,
    }
}

const fn showcase_class(expanded: bool) -> &'static str {
    if expanded {
        "showcase showcase--expanded"
    } else {
        "showcase showcase--collapsed"
    }
}

#[component]
fn ToggleButton(bus: ShowcaseBus) -> impl IntoView {
    view! {
        <button
            type="button"
            class="showcase__toggle"
            aria-expanded=move || bus.expanded.get()
            aria-controls="showcase-panel"
            on:click=move |_| {
                bus.expanded.update(|v| *v = !*v);
                bus.persist_expanded();
            }
        >
            <span class="showcase__toggle-icon" aria-hidden="true">
                {move || if bus.expanded.get() { "›" } else { "‹" }}
            </span>
            <span class="showcase__toggle-live" aria-hidden="true">"●"</span>
            <span class="showcase__toggle-label">"CODE"</span>
        </button>
    }
}

#[component]
fn Chrome(
    bus: ShowcaseBus,
    tab_files: RwSignal<Option<&'static [ShowcaseFile]>>,
    active_lang: RwSignal<ShowcaseLang>,
) -> impl IntoView {
    view! {
        <header class="showcase__chrome">
            <div class="showcase__heading">
                <span class="showcase__panel-name">{move || bus.active_panel.get()}</span>
                <span class="showcase__badge">{move || active_lang.get().label()}</span>
            </div>
            <div class="showcase__tab-groups" role="tablist" aria-label="Panel sources">
                {move || {
                    let files = tab_files.get().unwrap_or(&[]);
                    if files.is_empty() {
                        ().into_any()
                    } else {
                        let group_count = files.len() / TRINITY_LEN;
                        let multi = group_count > 1;

                        (0..group_count)
                            .map(|g| {
                                let start = g * TRINITY_LEN;
                                let group_files = &files[start..start + TRINITY_LEN];
                                let group_label = if multi {
                                    Some(group_label_for_stem(file_stem(group_files[0].name)))
                                } else {
                                    None
                                };

                                view! {
                                    <div class="showcase__tab-group">
                                        {group_label.map(|label| {
                                            view! {
                                                <span class="showcase__tab-group-label">
                                                    {label}
                                                </span>
                                            }
                                        })}
                                        <div class="showcase__tabs">
                                            {group_files
                                                .iter()
                                                .enumerate()
                                                .map(|(offset, file)| {
                                                    let idx = start + offset;
                                                    let stem =
                                                        truncate_middle(
                                                            file_stem(file.name),
                                                            TAB_STEM_MAX,
                                                        );
                                                    let ext = file_extension(file.name);
                                                    let title = file.name;
                                                    view! {
                                                        <button
                                                            type="button"
                                                            role="tab"
                                                            class=move || {
                                                                tab_class(
                                                                    bus.file_index.get() == idx,
                                                                )
                                                            }
                                                            aria-selected=move || {
                                                                bus.file_index.get() == idx
                                                            }
                                                            title=title
                                                            on:click=move |_| {
                                                                bus.file_index.set(idx);
                                                            }
                                                        >
                                                            <span class="showcase__tab-label">
                                                                <span class="showcase__tab-stem">
                                                                    {stem}
                                                                </span>
                                                                <span class="showcase__tab-ext">
                                                                    {"."}{ext}
                                                                </span>
                                                            </span>
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </header>
    }
}

const fn tab_class(active: bool) -> &'static str {
    if active {
        "showcase__tab showcase__tab--active"
    } else {
        "showcase__tab"
    }
}

#[component]
fn EditorPane(
    content: RwSignal<String>,
    active_lang: RwSignal<ShowcaseLang>,
    editor_fading: RwSignal<bool>,
    on_ready: Arc<dyn Fn(EditorHandle) + Send + Sync>,
) -> impl IntoView {
    view! {
        <div class="showcase__editor-wrap">
            <div
                class=move || {
                    if editor_fading.get() {
                        "showcase__editor showcase__editor--fading"
                    } else {
                        "showcase__editor"
                    }
                }
            >
                <CodeEditor
                    language=Signal::derive(move || active_lang.get().kode())
                    content=content.read_only()
                    theme=Signal::stored(Theme::tokyo_night())
                    on_ready=on_ready
                />
            </div>
            <div class="showcase__fade showcase__fade--bottom" aria-hidden="true"></div>
        </div>
    }
}

#[component]
fn StatusBar(bus: ShowcaseBus) -> impl IntoView {
    view! {
        <footer class="showcase__status">
            <span class="showcase__status-dot" aria-hidden="true"></span>
            <span class="showcase__status-text">{move || bus.status.get()}</span>
        </footer>
    }
}
