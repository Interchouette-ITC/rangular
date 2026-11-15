use std::path::PathBuf;

use rangular_aot::HeadComponentAdapter;

/// Build inputs and output locations for [`crate::build`].
///
/// Paths are relative to the Cargo package root. Component discovery checks
/// immediate subdirectories for a matching `<directory-name>.html` template
/// and optional `<directory-name>.scss` stylesheet.
///
/// Application-level document templates can be registered with
/// [`Self::document`], [`Self::head`], and [`Self::body`]. Each one is compiled
/// through the same AOT pipeline as components and generates
/// `OUT_DIR/rangular/<file-stem>_view.rs`.
///
/// # Examples
///
/// ```
/// use rangular_build::Config;
///
/// let config = Config::new("src/components", "style/components.css")
///     .global_scss("style/site.scss")
///     .document("src/document/document.html")
///     .head("src/document/head.html")
///     .body("src/document/body.html");
/// ```
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) components_dir: PathBuf,
    pub(crate) css_output: PathBuf,
    pub(crate) global_scss: Option<PathBuf>,
    pub(crate) document: Option<PathBuf>,
    pub(crate) head: Option<PathBuf>,
    pub(crate) body: Option<PathBuf>,
    pub(crate) head_adapter: Option<HeadComponentAdapter>,
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
            document: None,
            head: None,
            body: None,
            head_adapter: None,
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

    /// Register the application's document template.
    ///
    /// The template describes the complete document, typically
    /// `<!DOCTYPE html><html><head>…</head><body>…</body></html>`, and uses
    /// content projection for the parts supplied at render time. For example,
    /// `<ng-content select="head">` becomes a `slot_head: Children` parameter
    /// and a default `<ng-content>` becomes `children: Children`. Only a
    /// leading `<!DOCTYPE html>` declaration is accepted.
    ///
    /// # Arguments
    ///
    /// * `path` - Package-relative HTML file. The view is generated as
    ///   `<file-stem>_view` (`document.html` produces `document_view`). A
    ///   sibling `<file-stem>.scss` is compiled into the generated stylesheet.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::Config;
    ///
    /// let config = Config::new("src/components", "style/components.css")
    ///     .document("src/document/document.html");
    /// ```
    #[must_use]
    pub fn document(mut self, path: impl Into<PathBuf>) -> Self {
        self.document = Some(path.into());
        self
    }

    /// Register the application's document head template.
    ///
    /// The template is compiled with Rangular's head compiler, which supports
    /// `<title>`, `<meta>`, `<link>`, `<script>` and JSON-LD content and
    /// rejects other content with `RANG403`.
    ///
    /// # Arguments
    ///
    /// * `path` - Package-relative HTML file. The view is generated as
    ///   `<file-stem>_view` (`head.html` produces `head_view`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::Config;
    ///
    /// let config = Config::new("src/components", "style/components.css")
    ///     .head("src/document/head.html");
    /// ```
    #[must_use]
    pub fn head(mut self, path: impl Into<PathBuf>) -> Self {
        self.head = Some(path.into());
        self
    }

    /// Register the application's document body template.
    ///
    /// The template is compiled like a component template.
    ///
    /// # Arguments
    ///
    /// * `path` - Package-relative HTML file. The view is generated as
    ///   `<file-stem>_view` (`body.html` produces `body_view`). A sibling
    ///   `<file-stem>.scss` is compiled into the generated stylesheet.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::Config;
    ///
    /// let config = Config::new("src/components", "style/components.css")
    ///     .body("src/document/body.html");
    /// ```
    #[must_use]
    pub fn body(mut self, path: impl Into<PathBuf>) -> Self {
        self.body = Some(path.into());
        self
    }

    /// Compile the head template's `<title>`, `<meta>`, and `<link>` elements
    /// to caller-supplied Leptos components.
    ///
    /// Optional. Without an adapter, the head template emits native Leptos
    /// elements. Requires a template registered with [`Self::head`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rangular_build::{Config, HeadComponentAdapter};
    ///
    /// let config = Config::new("src/components", "style/components.css")
    ///     .head("src/document/head.html")
    ///     .head_adapter(HeadComponentAdapter::new(
    ///         "my_document::Title",
    ///         "my_document::Meta",
    ///         "my_document::Link",
    ///     ));
    /// ```
    #[must_use]
    pub fn head_adapter(mut self, adapter: HeadComponentAdapter) -> Self {
        self.head_adapter = Some(adapter);
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
        assert_eq!(configured.document, None);
        assert_eq!(configured.head, None);
        assert_eq!(configured.body, None);
        assert_eq!(configured.head_adapter, None);
    }

    #[test]
    fn stores_document_templates_and_head_adapter() {
        let adapter = rangular_aot::HeadComponentAdapter::new("a::T", "a::M", "a::L");
        let config = Config::new("components", "site.css")
            .document("doc/document.html")
            .head("doc/head.html")
            .body("doc/body.html")
            .head_adapter(adapter.clone());
        assert_eq!(config.document, Some(PathBuf::from("doc/document.html")));
        assert_eq!(config.head, Some(PathBuf::from("doc/head.html")));
        assert_eq!(config.body, Some(PathBuf::from("doc/body.html")));
        assert_eq!(config.head_adapter, Some(adapter));
    }
}
