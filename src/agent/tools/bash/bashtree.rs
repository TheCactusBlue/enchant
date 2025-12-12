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
            _ => false,
        }
    }

    pub fn is_allowed(&self, cfg: &BashConfig) -> bool {
        if !self.is_safe() {
            return false;
        }
        cfg.allow.iter().any(|rule| {
            let mut args: Vec<String> = rule.split(" ").map(|x| x.to_string()).collect();
            let program = args.remove(0);
            if program != self.program {
                return false;
            }
            // panic!("{:?}", (self, args));

            let wildcard = args.last().map(|x| x.as_str()) == Some("*");
            if wildcard {
                args.pop();
            }

            for (i, arg_rule) in args.iter().enumerate() {
                if Some(arg_rule) != self.args.get(i) {
                    return false;
                }
            }
            if !wildcard {
                return args.len() == self.args.len();
            }
            return true;
        })
    }
}
