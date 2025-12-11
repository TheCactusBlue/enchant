//! Minimal Bash AST used by Enchant.
//!
//! This is intentionally a **small subset** of bash suitable for parsing and
//! validating simple command lines before execution.
//!
//! Supported (initially):
//! - simple commands (words only; no redirects/assignments)
//! - pipelines: `a | b | c`
//! - and/or lists: `a && b`, `a || b`
//! - sequences: `a ; b` (and newlines)
//!
//! Not supported: subshells, compound commands, redirects, heredocs, async `&`,
//! `!`, `time`, process substitution, arithmetic, etc.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub first: Pipeline,
    pub rest: Vec<(AndOr, Pipeline)>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AndOr {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pipeline {
    pub commands: Vec<SimpleCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleCommand {
    pub program: String,
    pub args: Vec<String>,
}
