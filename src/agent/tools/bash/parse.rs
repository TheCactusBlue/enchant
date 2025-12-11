use brush_parser::{
    ParserOptions, SourceInfo,
    ast::{AndOr, Command, CompoundListItem, Pipeline, Program, SeparatorOperator},
};

use crate::error::Error;

pub fn parse_ast(input: &str) -> Result<Program, Error> {
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
    Ok(ast)
}

pub fn parse_statement(input: &str) -> Result<Vec<CompoundListItem>, Error> {
    let l = parse_ast(input)?
        .complete_commands
        .into_iter()
        .nth(0)
        .ok_or_else(|| {
            Error::BashError("Only bash operators that are permitted are &&, ||, |".to_string())
        })?
        .0;
    Ok(l)
}

pub fn parse_pipeline(pipeline: &Pipeline) -> Result<(), Error> {
    if pipeline.timed.is_some() || pipeline.bang {
        return Err(Error::BashError(
            "Negation or timed operators are unsupported".to_string(),
        ));
    }
    let pipeline = &pipeline.seq;
    let _res: Vec<_> = pipeline.iter().map(parse_command).collect();
    Ok(())
}

pub fn parse_command(command: &Command) -> Result<(), Error> {
    let command = if let Command::Simple(command) = command {
        command
    } else {
        return Err(Error::BashError(
            "Complex commands are not supported".to_string(),
        ));
    };
    dbg!(command);
    Ok(())
}

pub fn parse_bash_command(input: &str) -> Result<(), Error> {
    let statements = parse_statement(input)?;
    for statement in statements {
        match statement.1 {
            SeparatorOperator::Async => {
                return Err(Error::BashError(
                    "Async execution of bash commands are unsupported".to_string(),
                ));
            }
            _ => {}
        }
        let statement = statement.0;
        parse_pipeline(&statement.first)?;
        for pipeline in &statement.additional {
            let pipe = match pipeline {
                AndOr::And(pipeline) => parse_pipeline(pipeline),
                AndOr::Or(pipeline) => parse_pipeline(pipeline),
            }?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_2() {
        parse_bash_command("ls && pwd; echo 'hi there' | wc -l").unwrap()
    }
}
