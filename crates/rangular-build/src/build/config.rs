use std::path::PathBuf;

/// Build inputs and output locations for [`crate::build`].
///
/// Paths are relative to the Cargo package root. Component discovery checks
/// immediate subdirectories for a matching `<directory-name>.html` template
/// and optional `<directory-name>.scss` stylesheet.
///
/// # Examples
///
/// ```
/// use rangular_build::Config;
///
/// let config = Config::new("src/components", "style/components.css")
///     .global_scss("style/site.scss");
/// ```
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) components_dir: PathBuf,
    pub(crate) css_output: PathBuf,
    pub(crate) global_scss: Option<PathBuf>,
}

impl Config {
    /// Create a configuration with a component directory and generated
    /// stylesheet path.
    ///
    /// # Arguments
    ///
    /// * `components_dir` - Package-relative directory whose immediate child
    ///   directories contain matching `<name>.html` templates and optional
    ///   `<name>.scss` files. Commonly `src/components`.
    /// * `css_output` - Package-relative destination for the combined CSS.
    ///   Parent directories are created as needed; this path can point to a
    ///   stylesheet loaded by Trunk.
    ///
    /// The returned configuration has no global stylesheet. Add one with
    /// [`Self::global_scss`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::Config;
    ///
    /// let config = Config::new("src/components", "style/components.css");
    /// ```
    #[must_use]
    pub fn new(components_dir: impl Into<PathBuf>, css_output: impl Into<PathBuf>) -> Self {
        Self {
            components_dir: components_dir.into(),
            css_output: css_output.into(),
            global_scss: None,
        }
    }

    /// Set a global SCSS file to prepend to the generated stylesheet.
    ///
    /// # Arguments
    ///
    /// * `path` - Package-relative SCSS file. It is compiled before stylesheets
    ///   belonging to individual components.
    ///
    /// If the file is missing or contains invalid SCSS, [`crate::build`] reports
    /// the path and compiler diagnostics and panics, as expected in a Cargo
    /// build script.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::Config;
    ///
    /// let config = Config::new("src/components", "style/components.css")
    ///     .global_scss("style/site.scss");
    /// ```
    #[must_use]
    pub fn global_scss(mut self, path: impl Into<PathBuf>) -> Self {
        self.global_scss = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::path::PathBuf;

    #[test]
    fn stores_paths_and_optional_global_stylesheet() {
        let plain = Config::new("components", "style/generated.css");
        assert_eq!(plain.components_dir, PathBuf::from("components"));
        assert_eq!(plain.css_output, PathBuf::from("style/generated.css"));
        assert_eq!(plain.global_scss, None);

        let configured = plain.global_scss("style/global.scss");
        assert_eq!(
            configured.global_scss,
            Some(PathBuf::from("style/global.scss"))
        );
    }
}
