//! Compile-time embedded panel sources for the code showcase rail.

use kode_leptos::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowcaseLang {
    Html,
    Scss,
    Rust,
}

impl ShowcaseLang {
    pub const fn kode(self) -> Language {
        match self {
            Self::Html => Language::new_static("html"),
            Self::Scss => Language::new_static("css"),
            Self::Rust => Language::new_static("rust"),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Html => "Template",
            Self::Scss => "SCSS",
            Self::Rust => "Host",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShowcaseFile {
    pub name: &'static str,
    pub source: &'static str,
    pub lang: ShowcaseLang,
}

#[derive(Clone, Copy, Debug)]
pub struct ShowcaseEntry {
    pub panel_id: &'static str,
    pub files: &'static [ShowcaseFile],
}

macro_rules! panel_triple {
    ($dir:literal) => {
        [
            file(
                concat!($dir, ".html"),
                include_str!(concat!("../components/", $dir, "/", $dir, ".html")),
                ShowcaseLang::Html,
            ),
            file(
                concat!($dir, ".scss"),
                include_str!(concat!("../components/", $dir, "/", $dir, ".scss")),
                ShowcaseLang::Scss,
            ),
            file(
                concat!($dir, ".rs"),
                include_str!(concat!("../components/", $dir, "/", $dir, ".rs")),
                ShowcaseLang::Rust,
            ),
        ]
    };
}

pub const SHOWCASE: &[ShowcaseEntry] = &[
    entry("chrome-header", &panel_triple!("chrome_header")),
    entry("color-field", &panel_triple!("color_field")),
    entry("asset-icon", &panel_triple!("asset_icon")),
    entry("layout-shell", &panel_triple!("layout_shell")),
    entry("named-slots", &panel_triple!("named_slots")),
    entry(
        "io-parent",
        &[
            panel_triple!("io_parent")[0],
            panel_triple!("io_parent")[1],
            panel_triple!("io_parent")[2],
            panel_triple!("io_child")[0],
            panel_triple!("io_child")[1],
            panel_triple!("io_child")[2],
        ],
    ),
    entry("pipes", &panel_triple!("pipes")),
    entry("two-way", &panel_triple!("two_way")),
    entry("field-required", &panel_triple!("field_required")),
    entry("field-validators", &panel_triple!("field_validators")),
    entry("event-payload", &panel_triple!("event_payload")),
    entry("template-outlet", &panel_triple!("template_outlet")),
    entry("item-list", &panel_triple!("item_list")),
];

const fn entry(panel_id: &'static str, files: &'static [ShowcaseFile]) -> ShowcaseEntry {
    ShowcaseEntry { panel_id, files }
}

const fn file(name: &'static str, source: &'static str, lang: ShowcaseLang) -> ShowcaseFile {
    ShowcaseFile { name, source, lang }
}

pub fn entry_for(panel_id: &str) -> Option<&'static ShowcaseEntry> {
    SHOWCASE.iter().find(|e| e.panel_id == panel_id)
}

/// Default to the child template on io-parent (I/O bindings live there).
pub fn default_file_index(panel_id: &str) -> usize {
    if panel_id == "io-parent" {
        3
    } else {
        0
    }
}

pub fn source_for(
    panel_id: &str,
    file_index: usize,
) -> Option<(&'static [ShowcaseFile], &'static ShowcaseFile)> {
    let entry = entry_for(panel_id)?;
    let file = entry.files.get(file_index).unwrap_or(&entry.files[0]);
    Some((entry.files, file))
}

pub fn is_io_child_html(panel_id: &str, file_index: usize) -> bool {
    panel_id == "io-parent" && file_index == 3
}

pub fn file_extension(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(_, ext)| ext)
}

pub fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// Short group label when a panel exposes multiple component triples (e.g. io-parent).
pub fn group_label_for_stem(stem: &str) -> &str {
    stem.strip_prefix("io_").unwrap_or(stem)
}

/// Keeps both ends of a long label; extension stays on the suffix span in the UI.
pub fn truncate_middle(value: &str, max_len: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len();
    if len <= max_len {
        return value.to_owned();
    }
    if max_len <= 1 {
        return "…".to_owned();
    }
    let keep = max_len - 1;
    let front = keep / 2;
    let back = keep - front;
    let left: String = chars.iter().take(front).copied().collect();
    let right: String = chars.iter().skip(len - back).copied().collect();
    format!("{left}…{right}")
}

pub const TAB_STEM_MAX: usize = 12;

pub const TRINITY_LEN: usize = 3;
