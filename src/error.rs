use std::io;

use crate::agent::tools::tool_error::ToolError;

#[derive(Debug)]
pub enum Error {
    AIError(genai::Error),
    ToolError(ToolError),
    IOError(io::Error),
    SerdeError(String),
    BashError(String),
}

impl From<genai::Error> for Error {
    fn from(err: genai::Error) -> Self {
        Self::AIError(err)
    }
}

impl From<ToolError> for Error {
    fn from(err: ToolError) -> Self {
        Self::ToolError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err.to_string())
    }
}
