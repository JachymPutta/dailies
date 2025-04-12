use markdown::mdast::Node;
use markdown::mdast::Node::*;

pub fn replace_pattern(mdast: &mut Node, old: &str, new: &str) {
    unimplemented!()
}

// TODO: this should be more generic, pass through the whole ast and just
// update all occurences of title
// Update the title of the new entry with the current date
// fn update_title(template: &mut Node, config: &Config) {
//     match template {
//         Root(root) => {
//             for child in &mut root.children {
//                 update_title(child, config);
//             }
//         }
//         Heading(heading) => {
//             if let Text(text) = &mut heading.children[0] {
//                 if text.value == "{{title}}" {
//                     let cur_time = chrono::offset::Local::now();
//                     let cur_daily_name =
//                         format!("Daily: {}", cur_time.format(&config.name_template));
//                     text.value = cur_daily_name;
//                     // TODO: update position as well
//                 }
//             }
//         }
//         _ => (),
//     }
// }
