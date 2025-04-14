# dailies
Dailies is an extremely simple daily journal & habit tracker. It works on plain
markdown files, meaning it can incorporate well with existing tools/systems like
[Obsidian](https://obsidian.md/).

## Installation
### Nix
For nix users the package can be built with:
```sh
nix build
```

### Cargo
Building from source requires the Rust tool-chain. Once installed, either run
```sh
just build
```
or:
```sh
cargo build --release
```

## Usage
### Configuration
Dailies relies on a `.toml` configuration file which contains the following 
fields:
```toml
dailies_dir = "<DIR>" # Directory to save daily entries to
entry_template = "<PATH>" # Which template to use
prompt_path = "<PATH>" # List of prompts -- {{prompt}} will be replaced by a random one
date_template = "%Y-%m-%d" # Format used for date entries
```

Dailies will look for a configuration file in the following locations, in order:
1. `$HOME/.dailies.toml`
2. `$HOME/.config/dailies.toml`
3. `$HOME/.config/dailies/dailies.toml`
4. `$XDG_CONFIG_HOME/dailies.toml`
5. `$XDG_CONFIG_HOME/dailies/dailies.toml`
6. `$PWD/.dailies.toml`

### Template
The most important part of the configuration is the entry template. This is a
Markdown file that will be used to generate each daily entry. There are no 
requirements on the structure of the file, but `dailies` will look for several
sections:
- Header -- optionally the template can include a 'obsidian-style' header which
    will allow you to browse dailies within Obsidian, sample header may look like this:
```raw
---
id: "{{title}}"
tags:
- daily-notes
---

```
- `{{title}}` -- will be substituted for today's date according to the `date_template`
- `{{prompt}}` -- will be substituted for a random prompt from the `prompt_path`
- A heading labeled `Habits` followed by a list of `<name>: <streak>` will automatically 
    be incremented with each generated daily, to keep track of habit streaks
- A heading labeled `Todos` will get copied from the last daily to the current one 

> NOTE: There is a sample template in `examples/template.md`

### Calling dailies
Dailies can be run either directly from the command line or as a [nvim-plugin](https://github.com/JachymPutta/dailies.nvim).

