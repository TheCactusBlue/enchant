#[derive(Debug)]
pub enum Error {
    AIError(genai::Error),
}

impl From<genai::Error> for Error {
    fn from(err: genai::Error) -> Self {
        Self::AIError(err)
    }
}
