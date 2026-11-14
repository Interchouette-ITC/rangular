use std::fs;
use std::path::{Path, PathBuf};

use rangular_aot::HeadComponentAdapter;

use super::Config;
use super::component::relative_label;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Document,
    Head,
    Body,
}

impl Role {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Head => "head",
            Self::Body => "body",
        }
    }
}

/// A document-level template registered in [`Config`].
pub struct DocumentTemplate {
    pub role: Role,
    pub view_name: String,
    path: PathBuf,
}

impl DocumentTemplate {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Sibling `<stem>.scss` for document and body templates.
    pub fn scss_path(&self) -> Option<PathBuf> {
        if self.role == Role::Head {
            return None;
        }
        let path = self.path.with_extension("scss");
        path.is_file().then_some(path)
    }

    pub fn compile(
        &self,
        manifest: &Path,
        head_adapter: Option<&HeadComponentAdapter>,
    ) -> Result<String, String> {
        let source = relative_label(manifest, &self.path);
        let html = fs::read_to_string(&self.path).map_err(|error| {
            format!(
                "read {} template {}: {error}",
                self.role.label(),
                self.path.display()
            )
        })?;
        let compiled = match (self.role, head_adapter) {
            (Role::Head, Some(adapter)) => rangular_aot::compile_head_named_with_adapter(
                &html,
                &source,
                &self.view_name,
                adapter,
            ),
            (Role::Head, None) => rangular_aot::compile_head_named(&html, &source, &self.view_name),
            (Role::Document | Role::Body, _) => {
                rangular_aot::compile_named(&html, &source, &self.view_name)
            }
        };
        if !compiled.ok() {
            return Err(format!("{source}: {:?}", compiled.issues));
        }
        Ok(compiled.code)
    }
}

/// Resolves the registered document templates in `document`, `head`, `body` order.
pub fn resolve(config: &Config, manifest: &Path) -> Result<Vec<DocumentTemplate>, String> {
    if config.head_adapter.is_some() && config.head.is_none() {
        return Err("head_adapter requires a head template registered with Config::head".into());
    }
    [
        (Role::Document, &config.document),
        (Role::Head, &config.head),
        (Role::Body, &config.body),
    ]
    .into_iter()
    .filter_map(|(role, path)| path.as_ref().map(|path| (role, path)))
    .map(|(role, path)| {
        Ok(DocumentTemplate {
            role,
            view_name: view_name(role, path)?,
            path: manifest.join(path),
        })
    })
    .collect()
}

fn view_name(role: Role, path: &Path) -> Result<String, String> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("html") {
        return Err(format!(
            "{} template {} must be an `.html` file",
            role.label(),
            path.display()
        ));
    }
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let mut chars = stem.chars();
    let valid = chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric());
    if !valid {
        return Err(format!(
            "{} template {} must have a file stem that is a Rust identifier \
             (letters, digits, `_`) to generate `<stem>_view`",
            role.label(),
            path.display()
        ));
    }
    Ok(format!("{stem}_view"))
}

#[cfg(test)]
mod tests {
    use super::{Role, resolve, view_name};
    use crate::build::Config;
    use crate::test_support::TempDir;
    use rangular_aot::HeadComponentAdapter;
    use std::fs;
    use std::path::Path;

    #[test]
    fn view_names_follow_the_file_stem_convention() {
        assert_eq!(
            view_name(Role::Document, Path::new("src/document/document.html")).unwrap(),
            "document_view"
        );
        assert_eq!(
            view_name(Role::Body, Path::new("src/app_shell.html")).unwrap(),
            "app_shell_view"
        );
        for invalid in [
            "src/my-head.html",
            "src/1head.html",
            "src/.html",
            "src/head.htm",
        ] {
            assert!(
                view_name(Role::Head, Path::new(invalid)).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn resolves_registered_templates_in_role_order() {
        let manifest = Path::new("/pkg");
        let config = Config::new("components", "site.css")
            .body("doc/body.html")
            .document("doc/layout.html")
            .head("doc/head.html");
        let templates = resolve(&config, manifest).unwrap();
        let resolved: Vec<_> = templates
            .iter()
            .map(|t| (t.role, t.view_name.as_str(), t.path()))
            .collect();
        assert_eq!(
            resolved,
            [
                (
                    Role::Document,
                    "layout_view",
                    Path::new("/pkg/doc/layout.html")
                ),
                (Role::Head, "head_view", Path::new("/pkg/doc/head.html")),
                (Role::Body, "body_view", Path::new("/pkg/doc/body.html")),
            ]
        );
        assert!(
            resolve(&Config::new("components", "site.css"), manifest)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_a_head_adapter_without_a_head_template() {
        let config = Config::new("components", "site.css")
            .body("body.html")
            .head_adapter(HeadComponentAdapter::new("a::T", "a::M", "a::L"));
        let error = resolve(&config, Path::new("/pkg")).err().unwrap();
        assert!(error.contains("head_adapter requires a head template"));
    }

    #[test]
    fn only_document_and_body_templates_pick_up_sibling_scss() {
        let temp = TempDir::new();
        for name in ["layout", "head", "body"] {
            fs::write(temp.path().join(format!("{name}.html")), "<p>x</p>").unwrap();
            fs::write(
                temp.path().join(format!("{name}.scss")),
                "p { color: red; }",
            )
            .unwrap();
        }
        let config = Config::new("components", "site.css")
            .document("layout.html")
            .head("head.html")
            .body("body.html");
        let templates = resolve(&config, temp.path()).unwrap();
        assert_eq!(
            templates[0].scss_path(),
            Some(temp.path().join("layout.scss"))
        );
        assert_eq!(templates[1].scss_path(), None);
        assert_eq!(
            templates[2].scss_path(),
            Some(temp.path().join("body.scss"))
        );
    }
}
