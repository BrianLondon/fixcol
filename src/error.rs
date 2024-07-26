use std::io;

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

#[derive(Debug, Clone, Copy)]
pub struct DataError {
    line: Option<usize>,
}

impl DataError {
    pub(crate) fn new() -> Self {
        DataError {
            line: None,
        }
    }

    pub(crate) fn with_line(&self, line: usize) -> Self {
        let mut new_error = self.clone();
        new_error.line = Some(line);
        new_error
    }
}


