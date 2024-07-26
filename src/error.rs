use std::{io, num::{ParseFloatError, ParseIntError}};

#[derive(Debug)]
pub enum Error {
    DataError(DataError),
    IoError(io::Error)
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<DataError> for Error {
    fn from(value: DataError) -> Self {
        Self::DataError(value)
    }
}

#[derive(Debug, Clone)]
pub struct DataError {
    text: String,
    line: Option<usize>,
    inner_error: InnerError,
}

impl DataError {
    pub(crate) fn new(text: String) -> Self {
        DataError {
            text: text,
            line: None,
            inner_error: InnerError::None
        }
    }

    pub(crate) fn new_err<Err>(text: String, err: Err) -> Self 
        where Err: Into<InnerError>
    {
        DataError {
            text: text,
            line: None, 
            inner_error: err.into(),
        }
    }

    pub fn custom(parsed_value: String, message: String) -> Self {
        DataError {
            text: parsed_value,
            inner_error: InnerError::Custom(message),
            line: None,
        }
    }

    pub(crate) fn with_line(&self, line: usize) -> Self {
        let mut new_error = self.clone();
        new_error.line = Some(line);
        new_error
    }
}

#[derive(Debug, Clone)]
pub enum InnerError {
    None,
    Custom(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
}

impl From<ParseFloatError> for InnerError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}


impl From<ParseIntError> for InnerError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}