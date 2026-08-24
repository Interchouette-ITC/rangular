//! rangular: Angular-subset HTML and component CSS for Leptos on the web.
//!
//! Author templates in external `.html` / `.scss` files. Controllers stay
//! markup-free. See `SPEC.md` for the v0.1 language contract.

#![forbid(unsafe_code)]

mod registry;

pub use rangular_aot as aot;
pub use rangular_css as css;
pub use rangular_expr as expr;
pub use rangular_host as host;
pub use rangular_macros as macros;
pub use rangular_parser as parser;
pub use rangular_runtime as runtime;

pub use registry::{
    ComponentEntry, Registry, APP_ASSET_ICON, APP_CHROME_HEADER, APP_COLOR_FIELD, APP_IO_CHILD,
    APP_ITEM_LIST,
};

/// Workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
