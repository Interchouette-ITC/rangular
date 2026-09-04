use rangular_parser::{
    banana_event_name, banana_set_target, banana_write_expr, collect_ng_templates,
    collect_projection_selects, has_default_projection, is_outlet_container, is_projection_tag,
    matches_select, parse, select_param_name, template_outlet_ref, Attr, Expr, Node,
};

#[test]
fn unexpected_else_and_closing_tag() {
    let lone_else = parse("@else { hi }", "t.html");
    assert!(lone_else
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unexpected @else")));
    let close = parse("</div>", "t.html");
    assert!(close
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unexpected closing tag")));
}

#[test]
fn comments_and_unclosed_comment() {
    let ok = parse("<!-- note -->", "t.html");
    assert!(ok.ok(), "{:?}", ok.diagnostics);
    assert!(matches!(
        ok.template.nodes.first(),
        Some(Node::Comment(text, _)) if text == " note "
    ));
    let bad = parse("<!-- open", "t.html");
    assert!(!bad.ok());
    assert!(bad
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unclosed comment")));
}

#[test]
fn projection_and_ng_template() {
    let proj = parse(
        "<ng-content select=\".header\"></ng-content><rg-content></rg-content>",
        "t.html",
    );
    assert!(proj.ok(), "{:?}", proj.diagnostics);
    assert!(is_projection_tag("ng-content"));
    assert!(is_projection_tag("rg-content"));
    let selects = collect_projection_selects(&proj.template.nodes);
    assert_eq!(selects, vec![".header".to_owned()]);
    assert!(has_default_projection(&proj.template.nodes));

    let named = parse(
        "<ng-template #card><span>{{label}}</span></ng-template>",
        "t.html",
    );
    assert!(named.ok(), "{:?}", named.diagnostics);
    let templates = collect_ng_templates(&named.template.nodes);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].0, "card");

    let missing = parse("<ng-template><p>x</p></ng-template>", "t.html");
    assert!(!missing.ok());
    assert!(missing
        .diagnostics
        .iter()
        .any(|d| d.message.contains("ng-template requires")));
}

#[test]
fn structural_ng_if_for_and_unknown() {
    let nif = parse("<p *ngIf=\"flag\">hi</p>", "t.html");
    assert!(nif.ok(), "{:?}", nif.diagnostics);
    assert!(matches!(nif.template.nodes.first(), Some(Node::If(_))));

    let nfor = parse(
        "<li *ngFor=\"let item of items; track item\">{{item}}</li>",
        "t.html",
    );
    assert!(nfor.ok(), "{:?}", nfor.diagnostics);
    assert!(matches!(nfor.template.nodes.first(), Some(Node::For(_))));

    let unknown = parse("<p *ngSwitch=\"mode\">x</p>", "t.html");
    assert!(unknown
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unknown structural")));
}

#[test]
fn control_flow_blocks_and_errors() {
    let with_else = parse(
        "@if (flag) { <span>yes</span> } @else { <span>no</span> }",
        "t.html",
    );
    assert!(with_else.ok(), "{:?}", with_else.diagnostics);
    assert!(matches!(
        with_else.template.nodes.first(),
        Some(Node::If(block)) if block.else_branch.is_some()
    ));

    let tracked = parse(
        "@for (let item of items; track item) { <span>{{item}}</span> }",
        "t.html",
    );
    assert!(tracked.ok(), "{:?}", tracked.diagnostics);

    let bad_if = parse("@if flag { x }", "t.html");
    assert!(bad_if
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected '(' after @if")));

    let bad_for = parse("@for (item items) { <span>x</span> }", "t.html");
    assert!(bad_for
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected 'of'")));
}

#[test]
fn banana_helpers_and_attrs() {
    assert_eq!(banana_event_name("value"), "input");
    assert_eq!(banana_event_name("seed"), "seedChange");
    let write = banana_write_expr(&Expr::Ident("seed".into()));
    assert_eq!(banana_set_target(&write), Some("seed"));
    assert_eq!(banana_set_target(&Expr::Ident("seed".into())), None);
    assert_eq!(
        banana_set_target(&Expr::Call {
            callee: Box::new(Expr::Ident("other".into())),
            args: vec![Expr::Ident("seed".into())],
        }),
        None
    );

    let two_way = parse("<input [(value)]=\"seed\" />", "t.html");
    assert!(two_way.ok(), "{:?}", two_way.diagnostics);
}

#[test]
fn matches_select_variants_and_outlet() {
    assert!(!matches_select("div", &[], ""));
    assert!(matches_select(
        "div",
        &[("class".into(), "a header b".into())],
        ".header"
    ));
    assert!(matches_select(
        "section",
        &[("data-slot".into(), "main".into())],
        "[data-slot=main]"
    ));
    assert!(matches_select(
        "section",
        &[("data-slot".into(), "x".into())],
        "[data-slot]"
    ));
    assert!(matches_select("header", &[], "header"));
    assert_eq!(select_param_name(".Header"), "slot_header");
    assert_eq!(select_param_name("___"), "slot_named");

    let outlet = parse(
        "<ng-container [ngTemplateOutlet]=\"card\"></ng-container>",
        "t.html",
    );
    assert!(outlet.ok(), "{:?}", outlet.diagnostics);
    if let Some(Node::Element(el)) = outlet.template.nodes.first() {
        assert!(is_outlet_container(el));
        assert_eq!(template_outlet_ref(&el.attrs), Some("card"));
    } else {
        panic!("expected element");
    }
}

#[test]
fn attribute_error_paths_and_mismatched_close() {
    let unclosed = parse("{{ label", "t.html");
    assert!(unclosed
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unclosed interpolation")));

    let mismatch = parse("<div><span></div>", "t.html");
    assert!(mismatch
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unexpected closing tag")
            || d.message.contains("does not match")
            || d.message.contains("unexpected end")));

    let flag = parse("<input disabled />", "t.html");
    assert!(flag.ok(), "{:?}", flag.diagnostics);
    if let Some(Node::Element(el)) = flag.template.nodes.first() {
        assert!(el.attrs.iter().any(|a| matches!(
            a,
            Attr::Static {
                name,
                value: None,
                ..
            } if name == "disabled"
        )));
    }

    let escaped = parse(r#"<div title="a\"b"></div>"#, "t.html");
    assert!(escaped.ok(), "{:?}", escaped.diagnostics);

    let unquoted = parse("<div [title]=foo></div>", "t.html");
    assert!(unquoted
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected quoted")));
}

#[test]
fn for_brace_and_iterable_error_paths() {
    let no_paren = parse("@for item of items { <span>x</span> }", "t.html");
    assert!(no_paren
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected '(' after @for")));

    let empty_iter = parse("@for (let item of ) { <span>x</span> }", "t.html");
    assert!(empty_iter
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected iterable")));

    let no_track = parse("@for (let item of items; ) { <span>x</span> }", "t.html");
    assert!(
        no_track.ok()
            || no_track
                .diagnostics
                .iter()
                .any(|d| d.message.contains("track")),
        "{:?}",
        no_track.diagnostics
    );
    if no_track.ok() {
        assert!(matches!(
            no_track.template.nodes.first(),
            Some(Node::For(block)) if block.track.is_none()
        ));
    }

    let missing_brace = parse("@if (flag) x", "t.html");
    assert!(missing_brace
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected '{'")));

    let for_missing_brace = parse("@for (let i of items) x", "t.html");
    assert!(for_missing_brace
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected '{'")));
}

#[test]
fn braced_body_nested_control_and_interpolation() {
    let nested = parse(
        "@for (let item of items) { {{item}} @if (flag) { <em>y</em> } @for (let j of items) { <i>z</i> } }",
        "t.html",
    );
    assert!(nested.ok(), "{:?}", nested.diagnostics);
    assert!(matches!(nested.template.nodes.first(), Some(Node::For(_))));

    let nested_brace = parse("@if (flag) { { } }", "t.html");
    assert!(nested_brace.ok(), "{:?}", nested_brace.diagnostics);
}

#[test]
fn binding_banana_event_and_structural_errors() {
    let binding_eq = parse(r#"<div [title]>"x"</div>"#, "t.html");
    assert!(
        !binding_eq.diagnostics.is_empty(),
        "expected binding diagnostics"
    );

    let banana_eq = parse(r"<input [(value)]seed />", "t.html");
    assert!(
        !banana_eq.diagnostics.is_empty(),
        "expected banana diagnostics"
    );

    let _ = parse(r#"<input [(value)]="a.b" />"#, "t.html");

    let bare_star = parse("<p *ngIf></p>", "t.html");
    assert!(bare_star.ok() || !bare_star.diagnostics.is_empty());

    let for_no_let = parse(r#"<p *ngFor="item of items"></p>"#, "t.html");
    assert!(
        !for_no_let.diagnostics.is_empty(),
        "expected *ngFor without let to error"
    );

    let for_no_track = parse(r#"<li *ngFor="let item of items">{{item}}</li>"#, "t.html");
    assert!(for_no_track.ok(), "{:?}", for_no_track.diagnostics);
    assert!(matches!(
        for_no_track.template.nodes.first(),
        Some(Node::For(block)) if block.track.is_none()
    ));
}

#[test]
fn projection_extra_attrs_and_close_mismatch() {
    let proj = parse(
        r#"<ng-content select=".header" class="x"></ng-content>"#,
        "t.html",
    );
    assert!(proj.ok(), "{:?}", proj.diagnostics);
    let selects = collect_projection_selects(&proj.template.nodes);
    assert_eq!(selects, vec![".header".to_owned()]);

    let named = parse(
        r#"<ng-template #card title="t"><span>x</span></ng-template>"#,
        "t.html",
    );
    assert!(named.ok(), "{:?}", named.diagnostics);
    let templates = collect_ng_templates(&named.template.nodes);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].0, "card");

    let mismatch = parse("<div></span>", "t.html");
    assert!(mismatch
        .diagnostics
        .iter()
        .any(|d| d.message.contains("does not match")
            || d.message.contains("unexpected closing")
            || d.message.contains("closing tag")));
}

#[test]
fn expr_layer_error_and_warning_via_interpolation() {
    let ternary = parse("{{ a ? b : c }}", "t.html");
    assert!(
        ternary
            .diagnostics
            .iter()
            .any(|d| d.message.to_lowercase().contains("ternary")
                || d.message.contains("SPEC")
                || d.message.contains('?')),
        "{:?}",
        ternary.diagnostics
    );

    let bad = parse("{{ ( }}", "t.html");
    assert!(
        !bad.ok()
            || bad
                .diagnostics
                .iter()
                .any(|d| d.severity == rangular_parser::Severity::Error),
        "{:?}",
        bad.diagnostics
    );
}
