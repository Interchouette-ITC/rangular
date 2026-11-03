//! Cargo build-script support for compiling rangular templates and styles.

#![forbid(unsafe_code)]

mod build;

pub use build::{Config, build};
