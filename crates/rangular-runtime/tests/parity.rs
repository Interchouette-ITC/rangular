use rangular_aot::compile;
use rangular_host::{Host, HostError, Value};
use rangular_parser::{binding_ir, binding_ir_snapshot, parse};
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
    out.sort();
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

struct ParentHost {
    heading: String,
    child_label: String,
    muted: bool,
}

impl Host for ParentHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "heading" => Some(Value::Str(self.heading.clone())),
            "childLabel" => Some(Value::Str(self.child_label.clone())),
            "muted" => Some(Value::Bool(self.muted)),
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

struct ChromeHeaderHost {
    muted: bool,
    count_label: String,
    enabled_count: f64,
    total_count: f64,
    paused_count: f64,
}

impl Host for ChromeHeaderHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "muted" => Some(Value::Bool(self.muted)),
            "countLabel" => Some(Value::Str(self.count_label.clone())),
            "enabledCount" => Some(Value::Num(self.enabled_count)),
            "totalCount" => Some(Value::Num(self.total_count)),
            "pausedCount" => Some(Value::Num(self.paused_count)),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

struct ItemListHost {
    title: String,
    items: Vec<String>,
}

impl Host for ItemListHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "title" => Some(Value::Str(self.title.clone())),
            "items" => Some(Value::List(
                self.items.iter().cloned().map(Value::Str).collect(),
            )),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

#[test]
fn corpus_aot_runtime_ok_and_shared_ir() {
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

        let aot_ir = rangular_aot::structural_ir(&src, &file).expect("aot ir");
        let rt_ir = rangular_runtime::structural_ir(&src, &file).expect("runtime ir");
        assert_eq!(
            aot_ir.1, rt_ir.1,
            "{file}: structural IR must match between AOT and runtime wrappers"
        );
        assert!(
            !aot_ir.0.is_empty(),
            "{file}: expected non-empty structural IR"
        );
    }
}

#[test]
fn empty_template_same_rang401() {
    let aot = compile("", "empty_view");
    let mut host = EmptyHost;
    let rt = interpret("", "empty.html", &mut host);
    assert!(!aot.ok());
    assert!(!rt.ok());
    assert!(
        aot.issues.iter().any(|i| i.code == "RANG401"),
        "{:?}",
        aot.issues
    );
    assert!(
        rt.issues.iter().any(|i| i.code == "RANG401"),
        "{:?}",
        rt.issues
    );
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

    let parsed = parse(&src, "seed-bar.html");
    let ir = binding_ir_snapshot(&binding_ir(&parsed.template));
    assert!(ir.contains(r#"on:click="onGenerate""#), "{ir}");
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
fn chrome_header_runtime_snapshot() {
    let src =
        std::fs::read_to_string(fixture_root().join("components/chrome-header/chrome-header.html"))
            .unwrap();
    let mut host = ChromeHeaderHost {
        muted: true,
        count_label: "2/5".into(),
        enabled_count: 2.0,
        total_count: 5.0,
        paused_count: 3.0,
    };
    let out = interpret(&src, "chrome-header.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("2/5"), "{snap}");
    assert!(
        snap.contains(r#"class:chrome-header__mute-btn--active="true""#),
        "{snap}"
    );
    assert!(snap.contains(r#"attr:aria-pressed="true""#), "{snap}");
    assert!(snap.contains(r#"on:click="toggleMute""#), "{snap}");
    assert!(snap.contains("Unmute"), "{snap}");
    assert!(compile(&src, "chrome_header_view").ok());
}

#[test]
fn item_list_runtime_snapshot() {
    let src = std::fs::read_to_string(fixture_root().join("components/item-list/item-list.html"))
        .unwrap();
    let mut host = ItemListHost {
        title: "Picks".into(),
        items: vec!["hat".into(), "tie".into()],
    };
    let out = interpret(&src, "item-list.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("Picks"), "{snap}");
    assert!(snap.contains("hat"), "{snap}");
    assert!(snap.contains("tie"), "{snap}");
    assert!(compile(&src, "item_list_view").ok());

    let ir = rangular_aot::structural_ir(&src, "item-list.html")
        .unwrap()
        .1;
    assert!(ir.contains("@for item track"), "{ir}");
}

#[test]
fn event_payload_fixture_ir_and_aot() {
    let src = std::fs::read_to_string(fixture_root().join("html/event-payload.html")).unwrap();
    let ir = rangular_aot::structural_ir(&src, "event-payload.html")
        .unwrap()
        .1;
    assert!(ir.contains(r#"on:input="onInput""#), "{ir}");
    assert!(ir.contains(r#"on:click="onClick""#), "{ir}");
    assert!(ir.contains(r#"on:error="onError""#), "{ir}");
    assert!(compile(&src, "event_payload_view").ok());
    let mut host = EmptyHost;
    assert!(interpret(&src, "event-payload.html", &mut host).ok());
}

#[test]
fn layout_shell_has_ng_content_ir() {
    let src =
        std::fs::read_to_string(fixture_root().join("components/layout-shell/layout-shell.html"))
            .unwrap();
    let ir = rangular_aot::structural_ir(&src, "layout-shell.html")
        .unwrap()
        .1;
    assert!(ir.contains("ng-content"), "{ir}");
    assert!(compile(&src, "layout_shell_view").ok(), "AOT issues");
    let mut host = EmptyHost;
    assert!(interpret(&src, "layout-shell.html", &mut host).ok());
}

#[test]
fn io_parent_and_child_compile() {
    for rel in ["html/io-parent.html", "components/io-child/io-child.html"] {
        let src = std::fs::read_to_string(fixture_root().join(rel)).unwrap();
        assert!(
            compile(&src, "io_view").ok(),
            "{rel}: {:?}",
            compile(&src, "io_view").issues
        );
    }
}

#[test]
fn layout_shell_projects_runtime_slot() {
    let src =
        std::fs::read_to_string(fixture_root().join("components/layout-shell/layout-shell.html"))
            .unwrap();
    let mut host = EmptyHost;
    let slot = vec![rangular_runtime::VNode::Element {
        tag: "p".into(),
        attrs: vec![],
        children: vec![rangular_runtime::VNode::Text("panel".into())],
    }];
    let out = rangular_runtime::interpret_with_slot(&src, "layout-shell.html", &mut host, &slot);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("<p>"), "{snap}");
    assert!(snap.contains("panel"), "{snap}");
    assert!(
        snap.contains("layout-shell__stage")
            || snap.contains("class=\"layout-shell__stage\"")
            || snap.contains("stage"),
        "{snap}"
    );
}

#[test]
fn io_parent_classifies_inputs_and_outputs() {
    let src = std::fs::read_to_string(fixture_root().join("html/io-parent.html")).unwrap();
    let mut host = ParentHost {
        heading: "Alerts".into(),
        child_label: "Sounds".into(),
        muted: true,
    };
    let out = interpret(&src, "io-parent.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("Alerts"), "{snap}");
    assert!(snap.contains(r#"input:label="Sounds""#), "{snap}");
    assert!(snap.contains(r#"input:muted="true""#), "{snap}");
    assert!(snap.contains(r#"output:muteToggle="onMute""#), "{snap}");
    assert!(snap.contains("app-io-child"), "{snap}");

    let ir = rangular_aot::structural_ir(&src, "io-parent.html")
        .unwrap()
        .1;
    assert!(ir.contains("input:label"), "{ir}");
    assert!(ir.contains(r#"output:muteToggle="onMute""#), "{ir}");
    assert!(compile(&src, "io_parent_view").ok());
}

#[test]
fn pipes_runtime_snapshot() {
    struct PipesHost;

    impl Host for PipesHost {
        fn get(&self, name: &str) -> Option<Value> {
            match name {
                "label" => Some(Value::Str("Hello".into())),
                "amount" => Some(Value::Num(42.5)),
                _ => None,
            }
        }

        fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
            Ok(Value::Unit)
        }
    }

    let src = std::fs::read_to_string(fixture_root().join("html/pipes.html")).unwrap();
    let mut host = PipesHost;
    let out = interpret(&src, "pipes.html", &mut host);
    assert!(out.ok(), "{:?}", out.issues);
    let snap = out.snapshot();
    assert!(snap.contains("HELLO"), "{snap}");
    assert!(snap.contains("hello"), "{snap}");
    assert!(snap.contains("42.5"), "{snap}");
    assert!(snap.contains(r#""Hello""#), "{snap}");
    assert!(snap.contains(r#"attr:title="HELLO""#), "{snap}");
    assert!(compile(&src, "pipes_view").ok());

    let aot_ir = rangular_aot::structural_ir(&src, "pipes.html").expect("aot ir");
    let rt_ir = rangular_runtime::structural_ir(&src, "pipes.html").expect("rt ir");
    assert_eq!(aot_ir.1, rt_ir.1);
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
