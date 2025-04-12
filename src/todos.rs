use markdown::mdast::Node;
use markdown::mdast::Node::*;
use std::fs::write;
use std::path::PathBuf;

/// Move the TODOs from the last daily -- remove from previous, add to current
pub fn update_todos(template: &mut Node, previous: &mut Node, previous_date: PathBuf) {
    let todo_id = get_todo_id(template);
    if let (Root(root), Some(id)) = (template, todo_id) {
        let todos = get_todos_from_prev(previous);
        root.children.splice(id..id, todos.iter().cloned());

        let prev_output = mdast_util_to_markdown::to_markdown(previous).unwrap();
        write(previous_date, prev_output).unwrap();
    }
}

/// Find the element index in the root hierarchy where todos are
fn get_todo_id(node: &Node) -> Option<usize> {
    if let Root(root) = node {
        let children = &root.children;
        let len = children.len();
        for (i, child) in children.iter().enumerate() {
            if let Heading(heading) = child {
                if let Text(text) = &heading.children[0] {
                    if text.value == "Todos" && i < len {
                        return Some(i + 1);
                    }
                }
            }
        }
        None
    } else {
        None
    }
}

/// Collect all Todos from the previous daily, this will collect
/// everything from the "Todos" heading until the next heading of
/// the same or lower depth
fn get_todos_from_prev(node: &mut Node) -> Vec<Node> {
    let mut todos = vec![];
    let mut collecting = false;
    let mut priority = u8::MAX;
    if let Root(root) = node {
        let children = &mut root.children;
        let mut i = 0;
        while i < children.len() {
            if collecting {
                if let Heading(h) = &children[i] {
                    if h.depth <= priority {
                        break;
                    }
                }
                let cur = children.remove(i);
                todos.push(cur);
            }
            if let Heading(heading) = &children[i] {
                if let Text(text) = &heading.children[0] {
                    if text.value == "Todos" {
                        collecting = true;
                        priority = heading.depth;
                    }
                }
            }
            i += 1;
        }
        todos
    } else {
        todos
    }
}
