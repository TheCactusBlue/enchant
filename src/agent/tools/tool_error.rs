#[derive(Debug)]
pub enum ToolError {
    IOError(std::io::Error),
    Error { message: String },
}

impl From<std::io::Error> for ToolError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
