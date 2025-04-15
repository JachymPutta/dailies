use markdown::mdast::Node;
use markdown::mdast::Node::*;

/// Traverse the markdown ast and replace occurrences of a string
pub fn replace_pattern(node: &mut Node, old: &str, new: &str) {
    match node {
        Text(text) => text.value = text.value.replace(old, new),
        other => {
            if let Some(children) = other.children_mut() {
                for child in children {
                    replace_pattern(child, old, new);
                }
            }
        }
    }
}

pub fn mdast_to_string(node: &Node) -> String {
    let options = mdast_util_to_markdown::Options {
        bullet: '-',
        rule: '-',
        ..Default::default()
    };
    let output = mdast_util_to_markdown::to_markdown_with_options(node, &options).unwrap();

    // HACK: mdast_util_to_markdown escapes [ -- we manually remove the escape
    output.replace(r"\[", "[")
}
