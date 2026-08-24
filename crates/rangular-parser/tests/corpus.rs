use rangular_parser::{parse, Node};
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

#[test]
fn corpus_parses_without_errors() {
    for path in html_fixtures() {
        let src = std::fs::read_to_string(&path).unwrap();
        let file = path.to_string_lossy();
        let parsed = parse(&src, &file);
        assert!(
            parsed.ok(),
            "{file}: {:?}",
            parsed.errors().collect::<Vec<_>>()
        );
    }
}

#[test]
fn seed_bar_has_generate_binding() {
    let path = fixture_root().join("html/seed-bar.html");
    let src = std::fs::read_to_string(path).unwrap();
    let parsed = parse(&src, "seed-bar.html");
    assert!(parsed.ok());
    let section = parsed
        .template
        .nodes
        .iter()
        .find_map(|n| match n {
            Node::Element(el) if el.tag == "section" => Some(el),
            _ => None,
        })
        .expect("section");
    assert!(section.children.iter().any(|n| matches!(
        n,
        Node::Element(el) if el.tag == "button"
            && el.attrs.iter().any(|a| matches!(
                a,
                rangular_parser::Attr::Event { name, .. } if name == "click"
            ))
    )));
}

#[test]
fn asset_icon_has_nested_if() {
    let path = fixture_root().join("components/asset-icon/asset-icon.html");
    let src = std::fs::read_to_string(path).unwrap();
    let parsed = parse(&src, "asset-icon.html");
    assert!(parsed.ok());
    assert!(parsed
        .template
        .nodes
        .iter()
        .any(|n| matches!(n, Node::If(_))));
}

#[test]
fn color_field_has_for_loop() {
    let path = fixture_root().join("components/color-field/color-field.html");
    let src = std::fs::read_to_string(path).unwrap();
    let parsed = parse(&src, "color-field.html");
    assert!(parsed.ok());
    assert!(has_for(&parsed.template.nodes));
}

fn has_for(nodes: &[Node]) -> bool {
    nodes.iter().any(|n| match n {
        Node::For(_) => true,
        Node::Element(el) => has_for(&el.children),
        Node::If(b) => {
            has_for(&b.then_branch) || b.else_branch.as_ref().is_some_and(|e| has_for(e))
        }
        _ => false,
    })
}

#[test]
fn two_way_desugars_to_property_and_input_event() {
    let path = fixture_root().join("html/two-way.html");
    let src = std::fs::read_to_string(path).unwrap();
    let parsed = parse(&src, "two-way.html");
    assert!(parsed.ok(), "{:?}", parsed.diagnostics);
    let input = parsed
        .template
        .nodes
        .iter()
        .find_map(|n| match n {
            Node::Element(el) if el.tag == "section" => el.children.iter().find_map(|c| match c {
                Node::Element(child) if child.tag == "input" => Some(child),
                _ => None,
            }),
            _ => None,
        })
        .expect("input");
    assert!(input.attrs.iter().any(|a| matches!(
        a,
        rangular_parser::Attr::Property { name, .. } if name == "value"
    )));
    assert!(input.attrs.iter().any(|a| matches!(
        a,
        rangular_parser::Attr::Event { name, expr, .. }
            if name == "input" && rangular_parser::banana_set_target(expr) == Some("seed")
    )));
}

#[test]
fn garbage_input_never_panics() {
    for sample in [
        "<<<>",
        "{{{{",
        "@if (",
        "*ngIf=",
        "<div *unknown=\"x\">",
        "{{ muted ? 'a' : 'b' }}",
        "<span [broken",
    ] {
        let _ = parse(sample, "garbage.html");
    }
}

#[test]
fn unknown_directive_warns_not_panics() {
    let parsed = parse(r#"<div *unknown="yes"></div>"#, "warn.html");
    assert!(parsed.ok());
    assert!(parsed
        .diagnostics
        .iter()
        .any(|d| d.code == "RANG101" && d.message.contains("unknown structural directive")));
}
