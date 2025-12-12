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

use crate::util::enchant_config::BashConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    // the vec of commands are pipelines
    pub first: Vec<Command>,
    pub rest: Vec<(AndOr, Vec<Command>)>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AndOr {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

impl Expression {
    pub fn is_safe(&self) -> bool {
        self.first.iter().all(|cmd| cmd.is_safe())
            && self
                .rest
                .iter()
                .all(|(_, pipeline)| pipeline.iter().all(|cmd| cmd.is_safe()))
    }

    /// Returns true if **every** command in the expression is present in the allowlist.
    pub fn is_allowed(&self, cfg: &BashConfig) -> bool {
        self.first.iter().all(|cmd| cmd.is_allowed(cfg))
            && self
                .rest
                .iter()
                .all(|(_, pipeline)| pipeline.iter().all(|cmd| cmd.is_allowed(cfg)))
    }
}

impl Command {
    pub fn is_safe(&self) -> bool {
        match self.program.as_str() {
            "cat" | "cd" | "echo" | "false" | "grep" | "head" | "ls" | "nl" | "pwd" | "tail"
            | "true" | "wc" | "which" => true,
            "cargo" if self.args.get(0).map(String::as_str) == Some("check") => true,
            _ => false,
        }
    }

    pub fn is_allowed(&self, cfg: &BashConfig) -> bool {
        cfg.allow.iter().any(|rule| {
            let mut args: Vec<&str> = rule.split(" ").collect();
            let program = args.remove(0);
            program == &self.program
        })
    }
}
