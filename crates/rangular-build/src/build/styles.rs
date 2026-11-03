use std::fs;
use std::path::Path;

pub(crate) fn append(path: &Path, label: String, css: &mut String) -> Result<(), String> {
    let scss = fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let compiled = rangular_css::compile_scss(&scss);
    if !compiled.ok() {
        return Err(format!("{label}: {:?}", compiled.issues));
    }
    css.push_str(&compiled.css);
    css.push('\n');
    Ok(())
}
