use brush_parser::{ParserOptions, SourceInfo};

use crate::error::Error;

pub fn parse_bash(input: &str) -> Result<(), Error> {
    let tokens = brush_parser::tokenize_str(input).unwrap();
    let ast = brush_parser::parse_tokens(
        &tokens,
        &ParserOptions::default(),
        &SourceInfo {
            source: "<Bash Evaluator>".to_string(),
        },
    )
    .unwrap();
    todo!()
}
