use brush_parser::{
    ParserOptions, SourceInfo,
    ast::{AndOr, Command, CompoundListItem, Pipeline, Program, SeparatorOperator},
};

use crate::agent::tools::{
    bash::bashtree::{self},
    tool_error::ToolError,
};

pub fn parse_ast(input: &str) -> Result<Program, ToolError> {
    let tokens =
        brush_parser::tokenize_str(input).map_err(|err| ToolError::BashError(err.to_string()))?;
    let ast = brush_parser::parse_tokens(
        &tokens,
        &ParserOptions::default(),
        &SourceInfo {
            source: "<Bash Evaluator>".to_string(),
        },
    )
    .map_err(|err| ToolError::BashError(err.to_string()))?;
    Ok(ast)
}

pub fn parse_statements(input: &str) -> Result<Vec<CompoundListItem>, ToolError> {
    let command_list = parse_ast(input)?.complete_commands;

    if command_list.len() != 1 {
        return Err(ToolError::BashError(
            "Only bash operators that are permitted are &&, ||, |".to_string(),
        ));
    }

    let statements = command_list
        .into_iter()
        .nth(0)
        .ok_or_else(|| {
            ToolError::BashError("Only bash operators that are permitted are &&, ||, |".to_string())
        })?
        .0;
    Ok(statements)
}

pub fn parse_statement(input: &str) -> Result<CompoundListItem, ToolError> {
    let statements = parse_statements(input)?;
    if statements.len() != 1 {
        return Err(ToolError::BashError(
            "Only bash operators that are permitted are &&, ||, |".to_string(),
        ));
    }
    let statement = statements.into_iter().nth(0).ok_or_else(|| {
        ToolError::BashError("Only bash operators that are permitted are &&, ||, |".to_string())
    })?;
    if matches!(statement.1, SeparatorOperator::Async) {
        return Err(ToolError::BashError(
            "Async execution of bash commands are unsupported".to_string(),
        ));
    }
    Ok(statement)
}

pub fn parse_pipeline(pipeline: &Pipeline) -> Result<Vec<bashtree::Command>, ToolError> {
    if pipeline.timed.is_some() || pipeline.bang {
        return Err(ToolError::BashError(
            "Negation or timed operators are unsupported".to_string(),
        ));
    }
    let pipeline = &pipeline.seq;
    pipeline.iter().map(parse_command).collect()
}

pub fn parse_command(command: &Command) -> Result<bashtree::Command, ToolError> {
    let command = if let Command::Simple(command) = command {
        command
    } else {
        return Err(ToolError::BashError(
            "Complex commands are not supported".to_string(),
        ));
    };
    Ok(bashtree::Command {
        program: command.word_or_name.clone().unwrap().value,
        args: command.suffix.iter().map(|x| x.to_string()).collect(),
    })
}

pub fn parse_bash_expr(input: &str) -> Result<bashtree::Expression, ToolError> {
    let statement = parse_statement(input)?.0;

    let first = parse_pipeline(&statement.first)?;
    let mut rest = vec![];
    for x in statement.additional {
        rest.push(match x {
            AndOr::And(pipeline) => (bashtree::AndOr::And, parse_pipeline(&pipeline)?),
            AndOr::Or(pipeline) => (bashtree::AndOr::Or, parse_pipeline(&pipeline)?),
        });
    }

    Ok(bashtree::Expression { first, rest })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_2() {
        parse_bash_expr("ls && pwd || echo 'hi there' | wc -l").unwrap();
    }
}
