use markdown::mdast::Node;
use serde::Deserialize;
use std::env::{self, current_dir};
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub dailies_dir: PathBuf,
    pub entry_template: PathBuf,
    pub date_template: String,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::find_config_path();
        let config_raw = read_to_string(&config_path)
            .unwrap_or_else(|e| panic!("Error {:?} reading config: {:?}", e, &config_path));
        let config_: Config = toml::from_str(&config_raw).unwrap();
        Self::resolve_paths(config_, &config_path)
    }

    pub fn get_previous_daily(&self) -> Option<(Node, PathBuf, i32)> {
        let mut dailies_paths = read_dir(&self.dailies_dir)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();

        dailies_paths.sort();
        // TODO: check if the last is the current one -- if so return second to last
        dailies_paths.last().map(|path| {
            eprintln!("Last daily path: {:?}", &path);
            let cur_date = chrono::Local::now().date_naive();
            let prev_stem = path.file_stem().unwrap().to_string_lossy();
            let prev_date = chrono::NaiveDate::parse_from_str(&prev_stem, &self.date_template)
                .expect("Failed to parse previous date from file name");

            let duration = cur_date.signed_duration_since(prev_date);
            let days_since = duration.num_days() as i32;

            let prev_raw = read_to_string(path).unwrap();
            (
                markdown::to_mdast(&prev_raw, &markdown::ParseOptions::default()).unwrap(),
                path.clone(),
                days_since,
            )
        })
    }

    pub fn get_cur_daily_name(&self) -> String {
        let cur_time = chrono::offset::Local::now();
        format!("{}", cur_time.format(&self.date_template))
    }

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

    /// Look for an existing configuration file, if not found
    /// start the flow for creating one
    /// locations that will be checked are:
    /// $HOME/.dailies.toml
    /// $HOME/.config/dailies.toml
    /// $HOME/.config/dailies/dailies.toml
    /// $XDG_CONFIG_HOME/dailies.toml
    /// $XDG_CONFIG_HOME/dailies/dailies.toml
    /// $PWD/.dailies.toml
    fn find_config_path() -> PathBuf {
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
            eprintln!("Error; No configuration file found!");
            eprintln!("Refer to the ReadMe on how to create one");
            exit(1)
        }
    }
}
