//! Classify `[prop]` / `(event)` on registered component tags as Input / Output.

use std::collections::HashMap;

use crate::ast::{Attr, Element, Node, Template};

/// Input / output names for one component tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagIo {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl TagIo {
    #[must_use]
    pub fn new(inputs: &[&str], outputs: &[&str]) -> Self {
        Self {
            inputs: inputs.iter().map(|s| (*s).to_owned()).collect(),
            outputs: outputs.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

/// Builtin tags used by the fixture corpus (kept in sync with `Registry`).
#[must_use]
pub fn builtin_tag_io() -> HashMap<String, TagIo> {
    let mut map = HashMap::new();
    map.insert(
        "app-io-child".into(),
        TagIo::new(&["label", "muted"], &["muteToggle"]),
    );
    map.insert(
        "app-chrome-header".into(),
        TagIo::new(
            &[
                "muted",
                "countLabel",
                "enabledCount",
                "totalCount",
                "pausedCount",
            ],
            &["muteToggle"],
        ),
    );
    map
}

/// Rewrite Property→Input and Event→Output when the element tag is registered.
pub fn classify_bindings<S: ::std::hash::BuildHasher>(
    template: &mut Template,
    tags: &HashMap<String, TagIo, S>,
) {
    classify_nodes(&mut template.nodes, tags);
}

fn classify_nodes<S: ::std::hash::BuildHasher>(
    nodes: &mut [Node],
    tags: &HashMap<String, TagIo, S>,
) {
    for node in nodes {
        match node {
            Node::Element(el) => classify_element(el, tags),
            Node::If(block) => {
                classify_nodes(&mut block.then_branch, tags);
                if let Some(else_branch) = &mut block.else_branch {
                    classify_nodes(else_branch, tags);
                }
            }
            Node::For(block) => classify_nodes(&mut block.body, tags),
            Node::Text(_, _)
            | Node::Interpolation(_, _)
            | Node::Comment(_, _)
            | Node::Projection(_) => {}
        }
    }
}

fn classify_element<S: ::std::hash::BuildHasher>(
    el: &mut Element,
    tags: &HashMap<String, TagIo, S>,
) {
    if let Some(io) = tags.get(&el.tag) {
        el.attrs = el
            .attrs
            .drain(..)
            .map(|attr| match attr {
                Attr::Property { name, expr, span } if io.inputs.iter().any(|i| i == &name) => {
                    Attr::Input { name, expr, span }
                }
                Attr::Event { name, expr, span } if io.outputs.iter().any(|o| o == &name) => {
                    Attr::Output { name, expr, span }
                }
                other => other,
            })
            .collect();
    }
    classify_nodes(&mut el.children, tags);
}
