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
    let entries = fs::read_dir(root).map_err(|error| format!("read {}: {error}", root.display()))?;
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
