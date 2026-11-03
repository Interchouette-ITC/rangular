use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct Component {
    pub(crate) name: String,
    directory: PathBuf,
}

impl Component {
    pub(crate) fn scss_path(&self) -> Option<PathBuf> {
        let path = self.directory.join(format!("{}.scss", self.name));
        path.is_file().then_some(path)
    }
}

pub(crate) fn discover(root: &Path) -> Result<Vec<Component>, String> {
    let entries =
        fs::read_dir(root).map_err(|error| format!("read {}: {error}", root.display()))?;
    let mut components = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("read entry in {}: {error}", root.display()))?;
        let directory = entry.path();
        if !directory.is_dir() {
            continue;
        }

        let name = file_name(&directory)?;
        if directory.join(format!("{name}.html")).is_file() {
            components.push(Component { name, directory });
        }
    }
    components.sort_by(|left, right| left.name.cmp(&right.name));
    if components.is_empty() {
        return Err(format!("no components found under {}", root.display()));
    }
    Ok(components)
}

pub(crate) fn compile_template(
    component: &Component,
    manifest: &Path,
) -> Result<(String, String), String> {
    let html_path = component.directory.join(format!("{}.html", component.name));
    let html = fs::read_to_string(&html_path)
        .map_err(|error| format!("read {}: {error}", html_path.display()))?;
    let source = relative_label(manifest, &html_path);
    let view_name = format!("{}_view", component.name);
    let compiled = rangular_aot::compile_named(&html, &source, &view_name);
    if !compiled.ok() {
        return Err(format!("{source}: {:?}", compiled.issues));
    }
    Ok((view_name, compiled.code))
}

pub(crate) fn relative_label(manifest: &Path, path: &Path) -> String {
    path.strip_prefix(manifest)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("{} has no valid UTF-8 file name", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{Component, compile_template, discover, relative_label};
    use crate::test_support::TempDir;
    use std::fs;

    #[test]
    fn discovers_sorted_template_directories_and_optional_scss() {
        let temp = TempDir::new();
        let root = temp.path().join("components");
        fs::create_dir_all(root.join("zeta")).unwrap();
        fs::write(root.join("zeta/zeta.html"), "<p>Zeta</p>").unwrap();
        fs::write(root.join("zeta/zeta.scss"), ".zeta { color: red; }").unwrap();
        fs::create_dir_all(root.join("alpha")).unwrap();
        fs::write(root.join("alpha/alpha.html"), "<p>Alpha</p>").unwrap();
        fs::create_dir_all(root.join("ignored")).unwrap();
        fs::write(root.join("ignored/other.html"), "<p>Ignored</p>").unwrap();
        fs::write(root.join("file.txt"), "ignored").unwrap();

        let components = discover(&root).unwrap();
        assert_eq!(
            components
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        assert_eq!(components[0].scss_path(), None);
        assert_eq!(components[1].scss_path(), Some(root.join("zeta/zeta.scss")));
    }

    #[test]
    fn reports_missing_and_empty_component_roots() {
        let temp = TempDir::new();
        assert!(matches!(
            discover(&temp.path().join("absent")),
            Err(error) if error.contains("read ")
        ));
        let empty = temp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        assert!(matches!(discover(&empty), Err(error) if error.contains("no components found")));
    }

    #[test]
    fn compiles_template_and_reports_read_or_template_errors() {
        let temp = TempDir::new();
        let component_dir = temp.path().join("components/card");
        fs::create_dir_all(&component_dir).unwrap();
        let component = Component {
            name: "card".into(),
            directory: component_dir.clone(),
        };
        fs::write(component_dir.join("card.html"), "<p>Hello</p>").unwrap();
        let (name, code) = compile_template(&component, temp.path()).unwrap();
        assert_eq!(name, "card_view");
        assert!(!code.is_empty());

        fs::write(component_dir.join("card.html"), "{{ broken").unwrap();
        assert!(
            compile_template(&component, temp.path())
                .unwrap_err()
                .contains("card.html")
        );
        fs::remove_file(component_dir.join("card.html")).unwrap();
        assert!(
            compile_template(&component, temp.path())
                .unwrap_err()
                .contains("read ")
        );
    }

    #[test]
    fn relative_labels_handle_inside_and_outside_paths() {
        let temp = TempDir::new();
        assert_eq!(
            relative_label(temp.path(), &temp.path().join("src/a.html")),
            "src/a.html"
        );
        assert!(
            relative_label(temp.path(), std::path::Path::new("/outside/a.html"))
                .contains("outside/a.html")
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_component_directory_names_that_are_not_utf8() {
        use std::os::unix::ffi::OsStrExt;

        let temp = TempDir::new();
        let root = temp.path().join("components");
        fs::create_dir_all(&root).unwrap();
        let invalid_name = std::ffi::OsStr::from_bytes(b"bad-\xff");
        fs::create_dir(root.join(invalid_name)).unwrap();
        assert!(matches!(discover(&root), Err(error) if error.contains("valid UTF-8")));
    }
}
