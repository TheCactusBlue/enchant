use crate::agent::tools::tool::ToolError;

#[derive(Debug)]
pub enum Error {
    AIError(genai::Error),
    ToolError(ToolError),
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
