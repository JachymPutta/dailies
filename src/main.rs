use markdown::mdast::Node;
use markdown::mdast::Node::*;

use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{self, current_dir},
    fs::{read, read_dir, write},
    path::{Path, PathBuf},
    process,
};

#[derive(Deserialize, Debug)]
struct Config {
    dailies_dir: PathBuf,
    entry_template: PathBuf,
    name_template: String,
}

impl Config {
    fn resolve_paths(mut self, config_path: &Path) -> Self {
        let base = config_path.parent().unwrap_or_else(|| Path::new("."));
        if self.dailies_dir.is_relative() {
            self.dailies_dir = base.join(&self.dailies_dir);
        }
        if self.entry_template.is_relative() {
            self.entry_template = base.join(&self.entry_template);
        }
        self
    }
}

fn get_previous_daily(config: &Config) -> Option<(Node, PathBuf, i32)> {
    let mut dailies_paths = read_dir(&config.dailies_dir)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    dailies_paths.sort();
    dailies_paths.last().map(|path| {
        // eprintln!("Last daily path: {:?}", &path);
        let cur_date = chrono::Local::now().date_naive();
        let prev_stem = path.file_stem().unwrap().to_string_lossy();
        let prev_date = chrono::NaiveDate::parse_from_str(&prev_stem, &config.name_template)
            .expect("Failed to parse previous date from file name");

        let duration = cur_date.signed_duration_since(prev_date);
        let days_since = duration.num_days() as i32;

        let prev_raw = String::from_utf8(read(path).unwrap()).unwrap();
        (
            markdown::to_mdast(&prev_raw, &markdown::ParseOptions::default()).unwrap(),
            path.clone(),
            days_since,
        )
    })
}

/// Update the title of the new entry with the current date
fn update_title(template: &mut Node, config: &Config) {
    match template {
        Root(root) => {
            for child in &mut root.children {
                update_title(child, config);
            }
        }
        Heading(heading) => {
            if let Text(text) = &mut heading.children[0] {
                if text.value == "{{title}}" {
                    let cur_time = chrono::offset::Local::now();
                    let cur_daily_name =
                        format!("Daily: {}", cur_time.format(&config.name_template));
                    text.value = cur_daily_name;
                    // TODO: update position as well
                }
            }
        }
        _ => (),
    }
}

fn find_habit_list(node: &mut Node) -> Option<&mut Node> {
    if let Root(root) = node {
        let children = &mut root.children;
        let len = children.len();
        for (i, child) in children.iter_mut().enumerate() {
            if let Heading(heading) = child {
                if let Text(text) = &mut heading.children[0] {
                    if text.value == "Habits" && i < len {
                        // TODO children[i+1] need to be a list
                        return Some(&mut children[i + 1]);
                    }
                }
            }
        }
        None
    } else {
        None
    }
}

fn get_habit_map(node: &Node) -> HashMap<String, i32> {
    let mut habit_map = HashMap::new();
    fn traverse(cur: &Node, habit_map: &mut HashMap<String, i32>) {
        match cur {
            List(list) => {
                for node in list.children.iter() {
                    traverse(node, habit_map);
                }
            }
            ListItem(item) => {
                for node in item.children.iter() {
                    traverse(node, habit_map);
                }
            }
            Paragraph(par) => {
                if let Text(text) = &par.children[0] {
                    let text_raw_ = text.value.clone();
                    let text_split = text_raw_.split(':').collect::<Vec<_>>();
                    // TODO: what do we need to assert here? do we allow beyond habit: count?
                    assert!(text_split.len() >= 2);
                    let habit = text_split[0].into();
                    let streak = text_split[1].trim().parse().unwrap_or(0);
                    habit_map.insert(habit, streak);
                }
            }
            _ => unreachable!("process_habits: Unexpected node in List"),
        }
    }
    traverse(node, &mut habit_map);
    habit_map
}

fn update_habit_counters(node: &mut Node, map_: HashMap<String, i32>, days_since_last: i32) {
    // println!("{:?}", map_);
    fn traverse(cur: &mut Node, map: &HashMap<String, i32>, days_since_last: i32) {
        match cur {
            List(list) => {
                for node in list.children.iter_mut() {
                    traverse(node, map, days_since_last);
                }
            }
            ListItem(item) => {
                for node in item.children.iter_mut() {
                    traverse(node, map, days_since_last);
                }
            }
            Paragraph(par) => {
                if let Text(text) = &mut par.children[0] {
                    let habit = text.value.split(':').next().unwrap_or("MISSING");
                    if let Some((key, value)) = map.get_key_value(habit) {
                        let new_habit = format!("{}: {}", key, (value + days_since_last));
                        text.value = new_habit;
                    }
                }
            }
            _ => unreachable!("process_habits: Unexpected node in List"),
        }
    }
    traverse(node, &map_, days_since_last);
}

/// Increment all habits by one in the habit list
fn update_habits(template: &mut Node, previous: &mut Node, days_since_last: i32) {
    let templ_habits_ = find_habit_list(template);
    let prev_habits_ = find_habit_list(previous);

    if let (Some(templ_habits), Some(prev_habits)) = (templ_habits_, prev_habits_) {
        let prev_map = get_habit_map(prev_habits);
        update_habit_counters(templ_habits, prev_map, days_since_last);
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

/// Move the TODOs from the last daily -- remove from previous, add to current
fn update_todos(template: &mut Node, previous: &mut Node, previous_date: PathBuf) {
    let todo_id = get_todo_id(template);
    if let (Root(root), Some(id)) = (template, todo_id) {
        let todos = get_todos_from_prev(previous);
        root.children.splice(id..id, todos.iter().cloned());

        let prev_output = mdast_util_to_markdown::to_markdown(previous).unwrap();
        write(previous_date, prev_output).unwrap();
    }
}

/// Update the generic template based on the last entry
/// if there is no last entry, keep the generic template
fn update_template(config: &Config) -> String {
    // eprintln!("Reading template: {:?}", &config.entry_template);
    if let Ok(contents) = read(&config.entry_template) {
        let template_raw = String::from_utf8(contents).unwrap();
        if let Ok(mut parsed) =
            markdown::to_mdast(&template_raw, &markdown::ParseOptions::default())
        {
            update_title(&mut parsed, config);
            if let Some((mut previous_daily, previous_path, days_since_last)) =
                get_previous_daily(config)
            {
                update_habits(&mut parsed, &mut previous_daily, days_since_last);
                update_todos(&mut parsed, &mut previous_daily, previous_path);
            }

            let mut output = mdast_util_to_markdown::to_markdown(&parsed).unwrap();
            // TODO: Hacky -- handle this at the AST level
            output = output.replace(r"\[]", "[]");
            output
        } else {
            eprintln!(
                "ERROR: Parsing template failed. Is it valid markdown?: {:?}",
                &config.entry_template
            );
            process::exit(1)
        }
    } else {
        eprintln!("WARNING: Empty template: {:?}", &config.entry_template);
        "".into()
    }
}

/// Generate the next file name based on the name_template
/// specified in the configuration
fn generate_daily(config: &Config) {
    let cur_time = chrono::offset::Local::now();
    let cur_daily_name = format!("{}.md", cur_time.format(&config.name_template));
    // eprintln!("{:?}", cur_daily_name);
    let cur_daily_path = config.dailies_dir.join(PathBuf::from(cur_daily_name));
    // eprintln!("{:?}", cur_daily_path);

    if cur_daily_path.is_file() {
        // TODO: open the current daily in the $EDITOR
        // eprintln!("Today's daily already exists, exitting");
        // println!(
        //     "{}",
        //     String::from_utf8(read(&cur_daily_path).unwrap()).unwrap()
        // );
        let _ = update_template(&config);
    } else {
        let today_template = update_template(&config);
        // eprintln!("Writing to file: {:?}", &cur_daily_path);
        // println!("{}", today_template);
        write(&cur_daily_path, today_template)
            .unwrap_or_else(|_| panic!("Error writing to file: {:?}", &cur_daily_path));
    }
    println!("{:?}", cur_daily_path);
}

/// Create a new config.toml interactively
/// We need to configure the following attributes
/// dailies_dir: directory where dailies files are saved
/// entry_template: what a daily entry looks like
/// name_template: naming convention for files
fn _create_config() -> PathBuf {
    unimplemented!()
}

/// Look for an existing configuration file, if not found
/// start the flow for creating one
/// locations that will be checked are:
/// $HOME/.dailies.toml
/// $HOME/.config/dailies.toml
/// $HOME/.config/dailies/dailies.toml
/// $XDG_CONFIG_HOME/dailies.toml
/// $XDG_CONFIG_HOME/dailies/dailies.toml
/// $PWD/.dailies.toml
fn find_config() -> PathBuf {
    if let Ok(home_var) = env::var("HOME") {
        let home = PathBuf::from(&home_var);
        let home_dot = home.join(".dailies.toml");
        let home_conf_norm = home.join("config").join("dailies.toml");
        let home_conf_dir = home.join("config").join("dailies/dailies.toml");
        if home_dot.is_file() {
            return home_dot;
        } else if home_conf_norm.is_file() {
            return home_conf_norm;
        } else if home_conf_dir.is_file() {
            return home_conf_dir;
        }
    }

    if let Ok(xdg_home_var) = env::var("XDG_CONFIG_HOME") {
        let xdg_home = PathBuf::from(&xdg_home_var);
        let xdg_dot = xdg_home.join(".dailies.toml");
        let xdg_conf_norm = xdg_home.join("config").join("dailies.toml");
        let xdg_conf_dir = xdg_home.join("config").join("dailies/dailies.toml");

        if xdg_dot.is_file() {
            return xdg_dot;
        } else if xdg_conf_norm.is_file() {
            return xdg_conf_norm;
        } else if xdg_conf_dir.is_file() {
            return xdg_conf_dir;
        }
    }

    let pwd = current_dir().unwrap();
    let pwd_dot = pwd.join(".dailies.toml");
    if pwd_dot.is_file() {
        pwd_dot
    } else {
        // TODO: implement create_config() to make a config interactively
        eprintln!("Error; No configuration file found!");
        eprintln!("Refer to the ReadMe on how to create one");
        process::exit(1)
    }
}

fn main() {
    let config_path = find_config();
    let config_raw = String::from_utf8(
        read(&config_path)
            .unwrap_or_else(|e| panic!("Error {:?} reading config: {:?}", e, &config_path)),
    )
    .expect("Error reading config: Invalid characters present");
    let config_: Config = toml::from_str(&config_raw).unwrap();
    let config = config_.resolve_paths(&config_path);
    generate_daily(&config);
}
