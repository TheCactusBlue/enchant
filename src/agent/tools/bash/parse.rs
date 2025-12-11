use brush_parser::{ParserOptions, SourceInfo, ast::SeparatorOperator};

use crate::error::Error;

pub fn parse_bash(input: &str) -> Result<(), Error> {
    let tokens =
        brush_parser::tokenize_str(input).map_err(|err| Error::BashError(err.to_string()))?;
    let ast = brush_parser::parse_tokens(
        &tokens,
        &ParserOptions::default(),
        &SourceInfo {
            source: "<Bash Evaluator>".to_string(),
        },
    )
    .map_err(|err| Error::BashError(err.to_string()))?;
    let complete_commands = ast.complete_commands;
    let commands = complete_commands
        .into_iter()
        .nth(0)
        .ok_or_else(|| {
            Error::BashError("Only bash operators that are permitted are &&, ||, |".to_string())
        })?
        .0;

    let command = commands.into_iter().nth(0).ok_or_else(|| {
        Error::BashError("Only bash operators that are permitted are &&, ||, |".to_string())
    })?;

    match command.1 {
        SeparatorOperator::Async => {
            return Err(Error::BashError(
                "Async execution of bash commands are unsupported".to_string(),
            ));
        }
        _ => {}
    }

    // if complete_commands.len() != 1 {
    //     return Err(Error::BashError(
    //         "Only bash operators that are permitted are &&, ||, |".to_string(),
    //     ));
    // }
    // let compound_list = complete_commands.first();
    // if compound_list.len() != 1 {
    //     return Err(Error::BashError(
    //         "Only bash operators that are permitted are &&, ||, |".to_string(),
    //     ));
    // }
    // let second = first.get(0).unwrap().clone().0;

    todo!()
}
