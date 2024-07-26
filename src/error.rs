use std::fmt::{Display, Formatter};
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    DataError(DataError),
    IoError(io::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DataError(data_error) => data_error.fmt(f),
            Error::IoError(io_error) => io_error.fmt(f),
        }
    }
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

impl Error {
    pub fn from_utf8_error(bytes: &[u8], err: Utf8Error) -> Self {
        let (good_bytes, _) = bytes.split_at(err.valid_up_to());
        let text: String = String::from_utf8_lossy(good_bytes).into_owned();

        Self::DataError(DataError{
            text: text,
            line: None,
            inner_error: err.into(),
        })
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

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {     
        fn fmt_err(text: &String, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error decoding data \"{}\"\n", text)
        }

        match &self.inner_error {
            InnerError::None => fmt_err(&self.text, f)?,
            InnerError::Custom(s) => {
                fmt_err(&self.text, f)?;
                s.fmt(f)?;
            },
            InnerError::ParseIntError(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            },
            InnerError::ParseFloatError(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            },
            InnerError::Utf8Error(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            },
        }

        if let Some(line) = self.line {
            write!(f, "\nError occured on line {}", line)?;
        }

        write!(f, "\n")
    }
}

// TODO: Test case for invalid utf8 data

#[derive(Debug, Clone)]
pub enum InnerError {
    None,
    Custom(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    Utf8Error(Utf8Error),
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

impl From<Utf8Error> for InnerError {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}
