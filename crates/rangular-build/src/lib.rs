//! Cargo build-script support for compiling rangular HTML templates and SCSS.
//!
//! Configure the component directory and generated CSS path in `build.rs`, then
//! call [`build`] before the application crate is compiled. Each immediate
//! subdirectory containing a matching `<name>.html` file is compiled to an
//! includeable Rust view under Cargo's `OUT_DIR/rangular/`. A matching
//! `<name>.scss` file is compiled into the generated stylesheet.
//!
//! Application-level document templates are declared on the same [`Config`]
//! with [`Config::document`], [`Config::head`], and [`Config::body`]. They go
//! through the same AOT compiler and generate `<file-stem>_view.rs` next to the
//! component views; the head template uses the head compiler, optionally with a
//! [`HeadComponentAdapter`].
//!
//! # Example
//!
//! ```no_run
//! use rangular_build::{Config, build};
//!
//! fn main() {
//!     build(
//!         Config::new("src/components", "style/components.css")
//!             .global_scss("style/site.scss")
//!             .document("src/document/document.html")
//!             .head("src/document/head.html")
//!             .body("src/document/body.html"),
//!     );
//! }
//! ```

#![deny(unsafe_code)]

mod build;

#[cfg(test)]
mod test_support;

pub use build::{Config, build};
pub use rangular_aot::HeadComponentAdapter;
