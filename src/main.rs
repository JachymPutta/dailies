use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::process::exit;

mod config;
mod habit;
mod keywords;
mod mdast;
mod todos;

use crate::config::Config;
use crate::habit::update_habits;
use crate::keywords::*;
use crate::mdast::replace_pattern;
use crate::todos::update_todos;

/// Update the generic template based on the last entry
/// if there is no last entry, keep the generic template
fn update_template(config: &Config) -> String {
    // eprintln!("Reading template: {:?}", &config.entry_template);
    if let Ok(contents) = read_to_string(&config.entry_template) {
        if let Ok(mut parsed) = markdown::to_mdast(&contents, &markdown::ParseOptions::default()) {
            replace_pattern(&mut parsed, TITLE, &config.get_cur_daily_name());

            if let Some(prompt) = config.get_daily_prompt() {
                // println!("Found prompt: {}", prompt);
                replace_pattern(&mut parsed, PROMPT, &prompt);
            }

            if let Some((mut previous_daily, previous_path, days_since_last)) =
                config.get_previous_daily()
            {
                update_habits(&mut parsed, &mut previous_daily, days_since_last);
                update_todos(&mut parsed, &mut previous_daily, previous_path);
            }

            // println!("{:#?}", parsed);
            let mut output = mdast_util_to_markdown::to_markdown(&parsed).unwrap();

            // HACK: Handle this at the AST level
            output = output.replace(r"\[]", "[]");
            output = output.replace(r"***", "---"); // HACK: No idea why --- gets replaced with ***
            output
        } else {
            eprintln!(
                "ERROR: Parsing template failed. Is it valid markdown?: {:?}",
                &config.entry_template
            );
            exit(1)
        }
    } else {
        eprintln!("WARNING: Empty template: {:?}", &config.entry_template);
        "".into()
    }
}

/// Generate the next file name based on the date_template
/// specified in the configuration
fn generate_daily(config: &Config) {
    let cur_time = chrono::offset::Local::now();
    let cur_daily_name = format!("{}.md", cur_time.format(&config.date_template));
    // eprintln!("{:?}", cur_daily_name);
    let cur_daily_path = config.dailies_dir.join(PathBuf::from(cur_daily_name));
    // eprintln!("{:?}", cur_daily_path);

    if !cur_daily_path.is_file() {
        let today_template = update_template(config);
        // eprintln!("Writing to file: {:?}", &cur_daily_path);
        // println!("{}", today_template);
        write(&cur_daily_path, today_template)
            .unwrap_or_else(|_| panic!("Error writing to file: {:?}", &cur_daily_path));
    }
    println!("{:?}", cur_daily_path);
}

fn main() {
    let config = Config::load();
    generate_daily(&config);
}
