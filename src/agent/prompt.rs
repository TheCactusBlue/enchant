use std::collections::BTreeMap;

use handlebars::Handlebars;

pub fn build_system_prompt() -> String {
    let handlebars = Handlebars::new();

    let template = include_str!("../../prompts/BASE.md");
    let mut data = BTreeMap::new();
    data.insert(
        "workingDirectory".to_string(),
        std::env::current_dir().unwrap(),
    );

    handlebars.render_template(template, &data).unwrap()
}
