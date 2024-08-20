//! Type definitions and helpers for error handling in the `fixed` library.
//!
//! The main data type for this module is [`Error`] which will frequently be the
//! only type encountered by application authors, unless they are defining
//! custom deserialization logic, parsing nested types, etc. [`Error`] captures
//! deserialization errors encountered using `fixed` and many of the methods
//! provided by [`ReadFixed`] and [`WriteFixed`] return a [`Result<T, Error>`].
//!
//! Typical usage of the library in a command line application will have the
//! application print the error and exit. The error should have information
//! sufficient to identify where in the data file and on what data the error
//! occured.
//!
//! [`ReadFixed`]: crate::ReadFixed
//! [`WriteFixed`]: crate::WriteFixed
//!
//! # Example
//!
//! ```
//! use fixed::ReadFixed;
//! #[derive(ReadFixed)]
//! struct MyType {
//!     // Fields here
//! }
//!
//! use std::fs::File;
//! # fn f() {
//! let mut file = File::open("my_file.txt").unwrap();
//! for row in MyType::read_fixed_all(file) {
//!     match row {
//!         Ok(my_type) => {
//!             // Do something with my_type
//!         }
//!         Err(error) => {
//!             println!("{}", error);
//!             std::process::exit(1);
//!         }
//!     }
//! }
//! # }
//! ```
use std::fmt::{Display, Formatter};
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

/// The standard error for the `fixed` library.
///
/// `Error` captures both I/O errors and errors resulting from malformed inputs
/// that do not meet the expected format specification. Many of the methods
/// provided by [`ReadFixed`] and [`WriteFixed`] return a [`Result<T, Error>`].
/// While the `Error` contains structured data that can be used programatically
/// to identify what went wrong, it also can format (via [`to_string`]) to print
/// diagnostic errors to the user.
///
/// Note that there are factory methods that need to be public because they are
/// inserted by the derive macros, but should essentially never be used directly
/// by application authors.
///
/// # Example
///
/// ```
/// use fixed::ReadFixed;
/// #[derive(ReadFixed)]
/// struct MyType {
///     // Fields here
/// }
///
/// use std::fs::File;
/// # fn f() {
/// let mut file = File::open("my_file.txt").unwrap();
/// for row in MyType::read_fixed_all(file) {
///     match row {
///         Ok(my_type) => {
///             // Do something with my_type
///         }
///         Err(error) => {
///             println!("{}", error);
///             std::process::exit(1);
///         }
///     }
/// }
/// # }
/// ```
///
/// If the above example encounters an error it would print a message that
/// resembles the following.
///
/// ```text
/// Error decoding data from "123x6": invalid digit found in string
/// Error occured on line 56
/// ```
/// [`ReadFixed`]: crate::ReadFixed
/// [`WriteFixed`]: crate::WriteFixed
/// [`to_string`]: std::string::ToString::to_string()
///
#[derive(Debug)]
pub enum Error {
    /// An error that occured while parsing the formatted data
    DataError(DataError),
    /// An error that occured while reading or writing the data.
    ///
    /// This variant is a thin wrapper around [`std::io::Error`].
    IoError(io::Error),
}

impl Display for Error {
    /// Formats the error in a human-readable way.
    ///
    /// Creates a user facing diagnostic message to aid in troubleshooting a
    /// corrupted input or incorrectly annotated type with `#[derive(ReadFixed)]`.
    ///
    /// See [`Display::fmt`] docs for more information.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DataError(data_error) => data_error.fmt(f),
            Error::IoError(io_error) => io_error.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    /// Wraps an `std::io::Error` in a `fixed::error::Error`
    ///
    /// See [`From::from`] docs for more information.
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<DataError> for Error {
    /// Wraps an [`DataError`] in an [`Error`]
    ///
    /// See [`From::from`] docs for more information.
    fn from(value: DataError) -> Self {
        Self::DataError(value)
    }
}

impl From<FromUtf8Error> for Error {
    /// Wraps an [`FromUtf8Error`] in an [`Error`]
    ///
    /// This is used to wrap utf decoding errors and will preserve the byte
    /// sequence up to the location the error occured.
    ///
    /// See [`From::from`] docs for more information.
    fn from(value: FromUtf8Error) -> Self {
        Self::from_utf8_error(value)
    }
}

impl Error {
    /// Creates an `Error` from a `Utf8Error`
    fn from_utf8_error(inner: FromUtf8Error) -> Self {
        let bytes = inner.as_bytes();
        let err = inner.utf8_error();

        let (good_bytes, _) = bytes.split_at(err.valid_up_to());
        let text: String = String::from_utf8_lossy(good_bytes).into_owned();

        Self::DataError(DataError {
            text: text,
            line: None,
            inner_error: err.into(),
        })
    }

    pub fn unknown_key_error(key: String) -> Self {
        Self::DataError(DataError {
            text: key.to_owned(),
            line: None,
            inner_error: InnerError::UnknownKey,
        })
    }
}

/// Error indicating `fixed` failed to parse the supplied input
#[derive(Debug, Clone)]
pub struct DataError {
    text: String,
    line: Option<usize>,
    inner_error: InnerError,
}

impl DataError {
    pub(crate) fn new_err<Err>(text: String, err: Err) -> Self
    where
        Err: Into<InnerError>,
    {
        DataError {
            text: text,
            line: None,
            inner_error: err.into(),
        }
    }

    pub(crate) fn new_data_width_error(expected: usize, actual: usize) -> Self {
        DataError::new_err(
            format!(
                "Expected field to have width {} but supplied value has width {}.",
                expected,
                actual,
            ),
            InnerError::InvalidWidth(expected, actual)
        )
    }

    /// Creates a new custom `DataError`
    ///
    /// This method will typically be used when implementing custom deserialization
    /// logic through a [`FixedDeserializer`] implementation that also requires
    /// custom error handling to provide useful error messages.
    ///
    /// * `parsed_value` - The data that we failed to parse
    /// * `message` - A description of what went wrong
    ///
    /// [`FixedDeserializer`]: crate::FixedDeserializer
    ///
    /// # Example
    ///
    /// Consider a data file that contains a one character column with a nullable
    /// boolean (tri-state value) encoded as `'Y'` to mean true, `'N'` to mean
    /// false, or an empty column `' '` to mean null.
    ///
    /// We could create a new type with a custom parser to read that column and
    /// use `DataError::custom` to provide error context.
    ///
    /// ```
    /// use fixed::{FixedDeserializer, FieldDescription};
    /// use fixed::error::DataError;
    ///
    /// struct TriState(Option<bool>);
    ///
    /// impl FixedDeserializer for TriState {
    ///     fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<TriState, DataError> {
    ///         // We've defined this type as always having one column so confirm that
    ///         assert_eq!(desc.len, 1);
    ///         // burn columns we have to skip
    ///         let column = &s[desc.skip..desc.skip+1];
    ///         match column {
    ///             "Y" => Ok(TriState(Some(true))),
    ///             "N" => Ok(TriState(Some(false))),
    ///             " " => Ok(TriState(None)),
    ///             other => {
    ///                 Err(DataError::custom(other, "Expected \"Y\", \"N\", or an empty column"))
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn custom(parsed_value: &str, message: &str) -> Self {
        DataError {
            text: parsed_value.to_owned(),
            inner_error: InnerError::Custom(message.to_owned()),
            line: None,
        }
    }

    pub(crate) fn with_line(&self, line: usize) -> Self {
        let mut new_error = self.clone();
        new_error.line = Some(line);
        new_error
    }

    pub fn inner_error(&self) -> &InnerError {
        &self.inner_error
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn fmt_err(text: &String, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error decoding data from \"{}\": ", text)
        }

        match &self.inner_error {
            // InnerError::None => fmt_err(&self.text, f)?,
            InnerError::Custom(s) => {
                fmt_err(&self.text, f)?;
                s.fmt(f)?;
            }
            InnerError::ParseIntError(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            }
            InnerError::ParseFloatError(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            }
            InnerError::Utf8Error(e) => {
                fmt_err(&self.text, f)?;
                e.fmt(f)?;
            }
            InnerError::UnknownKey => {
                fmt_err(&self.text, f)?;
                write!(f, "Unrecognized enum key")?;
            }
            InnerError::InvalidWidth(exp, act) => {
                fmt_err(&self.text, f)?;
                write!(f, "Expected field of width {}. Found {}.", exp, act)?;
            }
        }

        if let Some(line) = self.line {
            write!(f, "\nError occured on line {}", line)?;
        }

        write!(f, "\n")
    }
}

/// Wrapper type for the known errors that can cause a [`DataError`].
#[derive(Debug, Clone)]
pub enum InnerError {
    Custom(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    Utf8Error(Utf8Error),
    UnknownKey,
    /// Params are expected len, actual len
    InvalidWidth(usize, usize),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_io_error() {
        fn need_error(_e: Error) -> bool {
            true
        }

        let io_error: io::Error = io::Error::new(io::ErrorKind::AlreadyExists, "uh oh");

        let b = need_error(io_error.into());
        assert!(b);
    }

    #[test]
    fn wrap_data_error() {
        fn need_error(_e: Error) -> bool {
            true
        }

        let data_error: DataError = DataError::new_err(
            "can't parse this".to_string(),
            InnerError::Custom("oh oh".to_string()),
        );

        let b = need_error(data_error.into());
        assert!(b);
    }

    #[test]
    fn wrap_utf8_error() {
        fn need_error(_e: Error) -> bool {
            true
        }

        let bytes: &[u8] = b"\x48\x69\xf0\x9f\x98\x89\xd1\x7b\x21";
        let res = String::from_utf8(bytes.to_vec()).map_err(|err| err.into());

        let b = need_error(res.unwrap_err());
        assert!(b);
    }

    #[test]
    fn utf8_error_text() {
        let bytes: &[u8] = b"\x48\x69\xf0\x9f\x98\x89\xd1\x7b\x21";
        let res: Error = String::from_utf8(bytes.to_vec())
            .map_err(|err| err.into())
            .unwrap_err();

        match res {
            Error::DataError(de) => {
                assert_eq!(de.text, String::from("Hi😉"));
                match de.inner_error {
                    InnerError::Utf8Error(inner) => {
                        assert_eq!(inner.valid_up_to(), 6);
                    }
                    _ => assert!(false),
                }
            }
            Error::IoError(_) => assert!(false),
        }
    }

    #[test]
    fn map_io_error() {
        use std::io::Write;

        fn try_write<W: Write>(buf: &mut W, bytes: &[u8]) -> Result<usize, Error> {
            let mut bytes_written = 0;
            // The conversion from io::Error to fixed::error::Error is what this
            // unit test is supposed to be evaluating
            bytes_written += buf.write(bytes)?;
            Ok(bytes_written)
        }

        // Test success
        let mut buf: Vec<u8> = Vec::new();
        let word: String = String::from("Hello!");

        let n_bytes = try_write(&mut buf, word.as_bytes()).unwrap();

        assert_eq!(n_bytes, 6);

        // Test failure
        struct FailedWritter {}

        impl Write for FailedWritter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::InvalidData, "failed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut buf = FailedWritter {};
        let word: String = String::from("1234567!");

        let res: Result<usize, Error> = try_write(&mut buf, word.as_bytes());

        match res {
            Err(Error::IoError(err)) => {
                assert_eq!(err.kind(), io::ErrorKind::InvalidData);
            }
            Err(Error::DataError(_)) => panic!("Expected IO Error"),
            Ok(_) => panic!("Expected IO Error"),
        };
    }
}
