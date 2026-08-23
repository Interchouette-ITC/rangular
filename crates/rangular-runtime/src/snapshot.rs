use crate::VNode;

/// Stable text form of a render tree for parity / golden tests.
#[must_use]
pub fn snapshot(nodes: &[VNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        write_node(node, 0, &mut out);
    }
    out
}

fn write_node(node: &VNode, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match node {
        VNode::Text(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                out.push_str(&pad);
                out.push_str(trimmed);
                out.push('\n');
            }
        }
        VNode::Element {
            tag,
            attrs,
            children,
        } => {
            out.push_str(&pad);
            out.push('<');
            out.push_str(tag);
            for (name, value) in attrs {
                out.push(' ');
                out.push_str(name);
                out.push('=');
                out.push('"');
                out.push_str(value);
                out.push('"');
            }
            if children.is_empty() {
                out.push_str("/>\n");
            } else {
                out.push_str(">\n");
                for child in children {
                    write_node(child, depth + 1, out);
                }
                out.push_str(&pad);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
        }
    }
}
