use std::path::PathBuf;

/// Inputs and output locations for [`crate::build`]. Paths are relative to the
/// Cargo package root unless stated otherwise.
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) components_dir: PathBuf,
    pub(crate) css_output: PathBuf,
    pub(crate) global_scss: Option<PathBuf>,
}

impl Config {
    /// Create a config with a component directory and generated stylesheet
    /// path. The component directory is usually `src/components` and the CSS
    /// output is often a path under `style/` consumed by Trunk.
    #[must_use]
    pub fn new(components_dir: impl Into<PathBuf>, css_output: impl Into<PathBuf>) -> Self {
        Self {
            components_dir: components_dir.into(),
            css_output: css_output.into(),
            global_scss: None,
        }
    }

    /// Add a global SCSS file to the start of the generated stylesheet.
    #[must_use]
    pub fn global_scss(mut self, path: impl Into<PathBuf>) -> Self {
        self.global_scss = Some(path.into());
        self
    }
}
