use markdown::mdast::Node;
use markdown::mdast::Node::*;

pub fn replace_pattern(mdast: &mut Node, old: &str, new: &str) {
    match mdast {
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
