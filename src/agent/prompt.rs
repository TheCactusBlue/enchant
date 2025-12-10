use std::{collections::BTreeMap, path::Path};

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

pub async fn read_enchant_md(working_directory: impl AsRef<Path>) -> Option<String> {
    // attempt to read from ENCHANT.md, AGENT.md, and CLAUDE.md in the working directory, in that order
    let dir = working_directory.as_ref();

    for filename in &["ENCHANT.md", "AGENT.md", "CLAUDE.md"] {
        let path = dir.join(filename);
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            return Some(content);
        }
    }

    None
}
