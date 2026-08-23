use rangular_aot::compile;
use rangular_host::{Host, HostError, Value};
use rangular_runtime::interpret;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn html_fixtures() -> Vec<PathBuf> {
    let root = fixture_root();
    let mut out = Vec::new();
    for entry in ["html", "components"] {
        let dir = root.join(entry);
        if !dir.is_dir() {
            continue;
        }
        if entry == "html" {
            for f in std::fs::read_dir(&dir).unwrap().flatten() {
                if f.path().extension().is_some_and(|e| e == "html") {
                    out.push(f.path());
                }
            }
        } else {
            for comp in std::fs::read_dir(&dir).unwrap().flatten() {
                let p = comp
                    .path()
                    .join(format!("{}.html", comp.file_name().to_string_lossy()));
                if p.is_file() {
                    out.push(p);
                }
            }
        }
    }
    out
}

struct EmptyHost;

impl Host for EmptyHost {
    fn get(&self, _: &str) -> Option<Value> {
        None
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

struct SeedBarHost {
    seed: String,
    worker_ready: bool,
    busy: bool,
    generated_for: String,
}

impl SeedBarHost {
    fn generate_disabled(&self) -> bool {
        !self.worker_ready
            || self.busy
            || (!self.generated_for.is_empty() && self.generated_for == self.seed)
    }

    const fn random_disabled(&self) -> bool {
        !self.worker_ready || self.busy
    }
}

impl Host for SeedBarHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.clone())),
            "generateDisabled" => Some(Value::Bool(self.generate_disabled())),
            "randomDisabled" => Some(Value::Bool(self.random_disabled())),
            "$event" => Some(Value::Str(String::new())),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

struct AssetIconHost {
    src: String,
    letter: String,
    letter_fallback: bool,
    size: String,
}

impl Host for AssetIconHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "src" => Some(Value::Str(self.src.clone())),
            "letter" => Some(Value::Str(self.letter.clone())),
            "letterFallback" => Some(Value::Bool(self.letter_fallback)),
            "size" => Some(Value::Str(self.size.clone())),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

struct ColorFieldHost {
    label: String,
    value: String,
    hex_draft: String,
    input_id: String,
    palette_open: bool,
    has_override: bool,
    swatches: Vec<String>,
    selected: String,
}

impl Host for ColorFieldHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "label" => Some(Value::Str(self.label.clone())),
            "value" => Some(Value::Str(self.value.clone())),
            "hexDraft" => Some(Value::Str(self.hex_draft.clone())),
            "inputId" => Some(Value::Str(self.input_id.clone())),
            "paletteOpen" => Some(Value::Bool(self.palette_open)),
            "hasOverride" => Some(Value::Bool(self.has_override)),
            "swatches" => Some(Value::List(
                self.swatches.iter().cloned().map(Value::Str).collect(),
            )),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        if name == "isSwatchSelected" {
            let hit = args.first().and_then(Value::as_str) == Some(self.selected.as_str());
            return Ok(Value::Bool(hit));
        }
        Ok(Value::Unit)
    }
}

#[test]
fn corpus_aot_and_runtime_both_ok() {
    for path in html_fixtures() {
        let src = std::fs::read_to_string(&path).unwrap();
        let file = path.to_string_lossy();
        let aot = compile(&src, "parity_view");
        let mut host = EmptyHost;
        let rt = interpret(&src, &file, &mut host);
        assert!(
            aot.ok() == rt.ok(),
            "{file}: aot.ok={} rt.ok={} aot={:?} rt={:?}",
            aot.ok(),
            rt.ok(),
            aot.issues,
            rt.issues
        );
        assert!(aot.ok(), "{file}: AOT {:?}", aot.issues);
        assert!(rt.ok(), "{file}: runtime {:?}", rt.issues);
    }
}

#[test]
fn seed_bar_runtime_snapshot() {
    let src = std::fs::read_to_string(fixture_root().join("html/seed-bar.html")).unwrap();
    let mut host = SeedBarHost {
        seed: "abc".into(),
        worker_ready: true,
        busy: false,
        generated_for: String::new(),
    };
    let out = interpret(&src, "seed-bar.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains(r#"prop:value="abc""#));
    assert!(snap.contains(r#"prop:disabled="false""#));
    assert!(snap.contains(r#"on:click="onGenerate""#));
    assert!(compile(&src, "seed_bar_view").ok());
}

#[test]
fn seed_bar_disabled_when_busy() {
    let src = std::fs::read_to_string(fixture_root().join("html/seed-bar.html")).unwrap();
    let mut host = SeedBarHost {
        seed: "abc".into(),
        worker_ready: true,
        busy: true,
        generated_for: String::new(),
    };
    let snap = interpret(&src, "seed-bar.html", &mut host).snapshot();
    assert!(snap.contains(r#"prop:disabled="true""#));
}

#[test]
fn asset_icon_letter_branch() {
    let src = std::fs::read_to_string(fixture_root().join("components/asset-icon/asset-icon.html"))
        .unwrap();
    let mut host = AssetIconHost {
        src: String::new(),
        letter: "A".into(),
        letter_fallback: true,
        size: "sm".into(),
    };
    let out = interpret(&src, "asset-icon.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("asset-icon__letter"));
    assert!(snap.contains('A'));
    assert!(!snap.contains("asset-icon__img"));
    assert!(compile(&src, "asset_icon_view").ok());
}

#[test]
fn asset_icon_src_branch() {
    let src = std::fs::read_to_string(fixture_root().join("components/asset-icon/asset-icon.html"))
        .unwrap();
    let mut host = AssetIconHost {
        src: "x.svg".into(),
        letter: "A".into(),
        letter_fallback: true,
        size: "md".into(),
    };
    let snap = interpret(&src, "asset-icon.html", &mut host).snapshot();
    assert!(snap.contains(r#"prop:src="x.svg""#));
    assert!(!snap.contains("asset-icon__letter"));
}

#[test]
fn color_field_for_expands_swatches() {
    let src =
        std::fs::read_to_string(fixture_root().join("components/color-field/color-field.html"))
            .unwrap();
    let mut host = ColorFieldHost {
        label: "Accent".into(),
        value: "#ff0000".into(),
        hex_draft: "#ff0000".into(),
        input_id: "c1".into(),
        palette_open: true,
        has_override: true,
        swatches: vec!["#f00".into(), "#0f0".into(), "#00f".into()],
        selected: "#0f0".into(),
    };
    let out = interpret(&src, "color-field.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("Accent"));
    assert!(snap.contains("attr:data-swatch=\"#f00\""));
    assert!(snap.contains("attr:data-swatch=\"#0f0\""));
    assert!(snap.contains("attr:data-swatch=\"#00f\""));
    assert!(snap.contains("class:color-field__swatch--selected=\"true\""));
    assert!(compile(&src, "color_field_view").ok());
}

#[test]
fn garbage_never_panics() {
    for sample in ["<<<>", "{{", "@if (", "<div *unknown=\"x\">"] {
        let aot = compile(sample, "broken");
        let mut host = EmptyHost;
        let rt = interpret(sample, "garbage.html", &mut host);
        let _ = (aot.ok(), rt.ok(), aot.code.len(), rt.snapshot());
    }
}
