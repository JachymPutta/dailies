use markdown::mdast::Node;
use markdown::mdast::Node::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Habit {
    pub name: String,
    pub count: i32,
}

impl Habit {
    pub fn from_line(s: &str) -> Option<Self> {
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() < 2 {
            return None;
        }
        Some(Habit {
            name: parts[0].trim().into(),
            count: parts[1].trim().parse().unwrap_or(0),
        })
    }
}

impl PartialEq for Habit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Habit {}

impl Hash for Habit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl std::fmt::Display for Habit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.count)
    }
}

/// Increment all habits by one in the habit list
pub fn update_habits(template: &mut Node, previous: &mut Node, days_since_last: i32) {
    let templ_habits_ = find_habit_list(template);
    let prev_habits_ = find_habit_list(previous);

    if let (Some(templ_habits), Some(prev_habits)) = (templ_habits_, prev_habits_) {
        let prev_map = get_habits(prev_habits);
        update_habit_counters(templ_habits, prev_map, days_since_last);
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

fn get_habits(node: &Node) -> HashSet<Habit> {
    let mut habits = HashSet::new();
    fn traverse(cur: &Node, habits: &mut HashSet<Habit>) {
        match cur {
            List(list) => {
                for node in list.children.iter() {
                    traverse(node, habits);
                }
            }
            ListItem(item) => {
                for node in item.children.iter() {
                    traverse(node, habits);
                }
            }
            Paragraph(par) => {
                if let Text(text) = &par.children[0] {
                    if let Some(habit) = Habit::from_line(&text.value) {
                        habits.insert(habit);
                    }
                }
            }
            _ => unreachable!("process_habits: Unexpected node in List"),
        }
    }
    traverse(node, &mut habits);
    habits
}

fn update_habit_counters(node: &mut Node, habits: HashSet<Habit>, days_since_last: i32) {
    // println!("{:?}", map_);
    fn traverse(cur: &mut Node, habits: &HashSet<Habit>, days_since_last: i32) {
        match cur {
            List(list) => {
                for node in list.children.iter_mut() {
                    traverse(node, habits, days_since_last);
                }
            }
            ListItem(item) => {
                for node in item.children.iter_mut() {
                    traverse(node, habits, days_since_last);
                }
            }
            Paragraph(par) => {
                if let Text(text) = &mut par.children[0] {
                    if let Some(habit_) = Habit::from_line(&text.value) {
                        if let Some(habit) = habits.get(&habit_) {
                            let new_habit = Habit {
                                name: habit.name.clone(),
                                count: habit.count + days_since_last,
                            };
                            text.value = new_habit.to_string();
                        }
                    }
                }
            }
            _ => unreachable!("process_habits: Unexpected node in List"),
        }
    }
    traverse(node, &habits, days_since_last);
}
