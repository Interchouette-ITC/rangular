use std::fs;
use std::path::Path;

pub(crate) fn append(path: &Path, label: String, css: &mut String) -> Result<(), String> {
    let scss =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let compiled = rangular_css::compile_scss(&scss);
    if !compiled.ok() {
        return Err(format!("{label}: {:?}", compiled.issues));
    }
    css.push_str(&compiled.css);
    css.push('\n');
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::append;
    use crate::test_support::TempDir;
    use std::fs;

    #[test]
    fn appends_compiled_styles_and_reports_read_and_compile_errors() {
        let temp = TempDir::new();
        let scss = temp.path().join("valid.scss");
        fs::write(&scss, ".card { color: red; }").unwrap();
        let mut css = String::new();
        append(&scss, "valid.scss".into(), &mut css).unwrap();
        assert!(css.contains(".card"));
        assert!(css.contains("color: red"));
        assert!(css.ends_with('\n'));

        let missing = temp.path().join("missing.scss");
        let error = append(&missing, "missing.scss".into(), &mut css).unwrap_err();
        assert!(error.contains("read "));

        let invalid = temp.path().join("invalid.scss");
        fs::write(&invalid, ".card { color: red;").unwrap();
        let error = append(&invalid, "invalid.scss".into(), &mut css).unwrap_err();
        assert!(error.contains("invalid.scss"));
    }
}
