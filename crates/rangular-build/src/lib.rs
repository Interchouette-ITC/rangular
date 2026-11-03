//! Cargo build-script support for compiling rangular HTML templates and SCSS.
//!
//! Configure the component directory and generated CSS path in `build.rs`, then
//! call [`build`] before the application crate is compiled. Each immediate
//! subdirectory containing a matching `<name>.html` file is compiled to an
//! includeable Rust view under Cargo's `OUT_DIR/rangular/`. A matching
//! `<name>.scss` file is compiled into the generated stylesheet.
//!
//! # Example
//!
//! ```no_run
//! use rangular_build::{Config, build};
//!
//! fn main() {
//!     build(
//!         Config::new("src/components", "style/components.css")
//!             .global_scss("style/site.scss"),
//!     );
//! }
//! ```

#![deny(unsafe_code)]

mod build;

#[cfg(test)]
mod test_support;

pub use build::{Config, build};
