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

pub async fn read_enchant_md() -> Option<String> {
    // attempt to read ENCHANT.md, AGENT.md, and CLAUDE.md
    todo!()
}
