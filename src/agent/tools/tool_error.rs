use std::fmt;

#[derive(Debug)]
pub enum ToolError {
    IOError(std::io::Error),
    Error { message: String },
    ToolNotFound,
    OutsideWorkingDirectory,
    BashError(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl From<std::io::Error> for ToolError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
