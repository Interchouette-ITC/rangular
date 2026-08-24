//! Projected `VNode`s and named/default projection bags.

use std::collections::HashMap;

use rangular_parser::matches_select;

#[derive(Clone, Debug, PartialEq)]
pub enum VNode {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Self>,
    },
    Text(String),
}

/// Projected roots partitioned into named selects and a default bucket.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectionBag {
    pub default: Vec<VNode>,
    pub named: HashMap<String, Vec<VNode>>,
}

impl ProjectionBag {
    #[must_use]
    pub fn from_flat(roots: &[VNode], selects: &[String]) -> Self {
        if selects.is_empty() {
            return Self {
                default: roots.to_vec(),
                named: HashMap::new(),
            };
        }
        let mut used = vec![false; roots.len()];
        let mut named = HashMap::new();
        for select in selects {
            let mut matched = Vec::new();
            for (i, root) in roots.iter().enumerate() {
                if used[i] {
                    continue;
                }
                if vnode_matches(root, select) {
                    matched.push(root.clone());
                    used[i] = true;
                }
            }
            named.insert(select.clone(), matched);
        }
        let default = roots
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(_, v)| v.clone())
            .collect();
        Self { default, named }
    }

    #[must_use]
    pub fn for_select(&self, select: Option<&str>) -> &[VNode] {
        select.map_or(self.default.as_slice(), |sel| {
            self.named.get(sel).map_or(&[], Vec::as_slice)
        })
    }
}

fn vnode_matches(node: &VNode, select: &str) -> bool {
    match node {
        VNode::Element { tag, attrs, .. } => matches_select(tag, attrs, select),
        VNode::Text(_) => false,
    }
}
