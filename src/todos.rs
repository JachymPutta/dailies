use crate::mdast::mdast_to_string;
use markdown::mdast::Node;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

/// Move the TODOs from the last daily -- remove from previous, add to current
pub fn update_todos(template: &Node, previous_date: &PathBuf) -> String {
    let template_str = mdast_to_string(template);
    let prev_str = read_to_string(previous_date).unwrap();
    let (prev, todos) = remove_todos_section(&prev_str);
    write(previous_date, prev).unwrap();
    insert_todos_section(&template_str, &todos)
}

// HACK: this should really be handled at the AST level
fn remove_todos_section(markdown: &str) -> (String, String) {
    let mut main_content = String::new();
    let mut todos_section = String::new();
    let mut skipping = false; // Indicates we're inside the todos section.
    let mut todos_level: Option<usize> = None; // Will hold the header level for the todos section.

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if !skipping {
            if trimmed.starts_with('#') {
                let header_level = trimmed.chars().take_while(|&c| c == '#').count();
                let header_text = trimmed[header_level..].trim_start();
                if header_text.to_lowercase().starts_with("todos") {
                    skipping = true;
                    todos_level = Some(header_level);
                    main_content.push_str(line);
                    main_content.push('\n');
                    continue;
                }
            }
            main_content.push_str(line);
            main_content.push('\n');
        } else {
            if trimmed.starts_with('#') {
                let header_level = trimmed.chars().take_while(|&c| c == '#').count();
                if let Some(level) = todos_level {
                    if header_level <= level {
                        skipping = false;
                        main_content.push_str(line);
                        main_content.push('\n');
                        continue;
                    }
                }
            }
            todos_section.push_str(line);
            todos_section.push('\n');
        }
    }

    (main_content, todos_section)
}

// HACK: this should really be handled at the AST level
fn insert_todos_section(base_markdown: &str, new_todos_content: &str) -> String {
    let mut result = String::new();
    let mut in_empty_todos = false;
    let mut todos_level: Option<usize> = None;
    let mut inserted = false;

    for line in base_markdown.lines() {
        let trimmed = line.trim_start();
        if !in_empty_todos {
            if trimmed.starts_with('#') {
                let header_level = trimmed.chars().take_while(|&c| c == '#').count();
                let header_text = trimmed[header_level..].trim_start();
                if header_text.to_lowercase().starts_with("todos") {
                    todos_level = Some(header_level);
                    in_empty_todos = true;
                    result.push_str(line);
                    result.push('\n');
                    result.push_str(new_todos_content);
                    result.push('\n');
                    inserted = true;
                    continue;
                }
            }
            result.push_str(line);
            result.push('\n');
        } else if trimmed.starts_with('#') {
            let header_level = trimmed.chars().take_while(|&c| c == '#').count();
            if let Some(level) = todos_level {
                if header_level <= level {
                    in_empty_todos = false;
                    result.push_str(line);
                    result.push('\n');
                    continue;
                }
            }
        }
    }
    if !inserted {
        result.push_str(new_todos_content);
    }
    result
}
