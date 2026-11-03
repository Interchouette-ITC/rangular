//! Cargo build-script support for compiling rangular templates and styles.

#![deny(unsafe_code)]

mod build;

#[cfg(test)]
mod test_support;

pub use build::{Config, build};
