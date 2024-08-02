extern crate fixed;
extern crate fixed_derive;

use std::io::{Error as IoError, ErrorKind, Write};

use fixed::error::Error;
use fixed::WriteFixedAll;
use fixed_derive::WriteFixed;

/// A writable buffer that accepts a maximum number of bytes and then errors
///
/// It's useful when you want to test a system's response to I/O errors
struct FakeBuffer {
    max_size: usize,
    data_size: usize,
    data: Vec<u8>,
}

impl FakeBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            data_size: 0,
            data: Vec::new(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data.as_slice()
    }

    pub fn as_string(&self) -> Option<String> {
        std::str::from_utf8(&self.as_slice())
            .ok()
            .map(|x| x.to_string())
    }
}

impl Write for FakeBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.data_size + buf.len() <= self.max_size {
            let written = self.data.write(buf).unwrap();
            self.data_size += written;
            Ok(written)
        } else {
            let err = IoError::new(ErrorKind::WriteZero, "Out of space");
            Err(err)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

//
// Tests of struct writes
//

const EXPECTED_STRUCT_TEXT: &'static str = r#"91 115
221159
92 0  
151171
"#;

#[derive(Debug, Eq, PartialEq, WriteFixed)]
struct Point {
    #[fixed(width = 3)]
    x: u8,
    #[fixed(width = 3)]
    y: u8,
}

impl Point {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub fn sample() -> Vec<Self> {
        vec![
            Point::new(91, 115),
            Point::new(221, 159),
            Point::new(92, 0),
            Point::new(151, 171),
        ]
    }
}

#[test]
fn struct_normal_buffer_control() {
    let points = Point::sample();

    let mut buf: Vec<u8> = Vec::new();
    let res = points.write_fixed_all(&mut buf);

    let text = std::str::from_utf8(buf.as_slice()).unwrap().to_string();

    assert!(res.is_ok());
    assert_eq!(text, EXPECTED_STRUCT_TEXT);
}

#[test]
fn struct_adequate_size_control() {
    let points = Point::sample();

    let mut buf = FakeBuffer::new(50);
    let res = points.write_fixed_all(&mut buf);

    let text = buf.as_string().unwrap();

    assert!(res.is_ok());
    assert_eq!(text, EXPECTED_STRUCT_TEXT);
}

#[test]
fn struct_out_of_space_test() {
    let points = Point::sample();

    let mut buf = FakeBuffer::new(20);
    let res = points.write_fixed_all(&mut buf);

    assert!(res.is_err());

    match res.unwrap_err() {
        Error::DataError(_) => panic!("Should have had I/O Error"),
        Error::IoError(e) => {
            assert_eq!(e.to_string(), "Out of space");
            assert_eq!(e.kind(), ErrorKind::WriteZero);
        }
    }
}

//
// Test of enum writes
//

#[derive(Debug, Eq, PartialEq, WriteFixed)]
#[fixed(key_width = 1)]
enum Datum {
    #[fixed(key = "S")]
    Scalar(#[fixed(width = 10, align = "right")] u16),
    #[fixed(key = "P")]
    Pair {
        #[fixed(width = 5)]
        x: u16,
        #[fixed(width = 5)]
        y: u16,
    },
    #[fixed(key = "U")]
    Unit,
}

impl Datum {
    pub fn sample() -> Vec<Self> {
        vec![
            Datum::Pair { x: 53542, y: 72 },
            Datum::Unit,
            Datum::Unit,
            Datum::Scalar(1234),
        ]
    }
}

const EXPECTED_ENUM_TEXT: &'static str = r#"P5354272   
U
U
S      1234
"#;

#[test]
fn enum_normal_buffer_control() {
    let data = Datum::sample();

    let mut buf: Vec<u8> = Vec::new();
    let res = data.write_fixed_all(&mut buf);

    let text = std::str::from_utf8(buf.as_slice()).unwrap().to_string();

    assert!(res.is_ok());
    assert_eq!(text, EXPECTED_ENUM_TEXT);
}

#[test]
fn enum_adequate_size_control() {
    let data = Datum::sample();

    let mut buf = FakeBuffer::new(50);
    let res = data.write_fixed_all(&mut buf);

    let text = buf.as_string().unwrap();

    assert!(res.is_ok());
    assert_eq!(text, EXPECTED_ENUM_TEXT);
}

#[test]
fn out_of_space_in_struct_variant() {
    let data = Datum::sample();

    let mut buf = FakeBuffer::new(7);
    let res = data.write_fixed_all(&mut buf);

    assert!(res.is_err());

    match res.unwrap_err() {
        Error::DataError(_) => panic!("Should have had I/O Error"),
        Error::IoError(e) => {
            assert_eq!(e.to_string(), "Out of space");
            assert_eq!(e.kind(), ErrorKind::WriteZero);
        }
    }

    // Confirm we failed in the struct variant
    let expected = "P53542";
    assert_eq!(buf.as_string().unwrap(), expected);
}

#[test]
fn out_of_space_in_tuple_variant() {
    let data = Datum::sample();

    let mut buf = FakeBuffer::new(15);
    let res = data.write_fixed_all(&mut buf);

    assert!(res.is_err());

    match res.unwrap_err() {
        Error::DataError(_) => panic!("Should have had I/O Error"),
        Error::IoError(e) => {
            assert_eq!(e.to_string(), "Out of space");
            assert_eq!(e.kind(), ErrorKind::WriteZero);
        }
    }

    // Confirm we failed in the tuple variant
    let expected = "P5354272   \nU\nU";
    assert_eq!(buf.as_string().unwrap(), expected);
}

#[test]
fn out_of_space_in_unit_variant() {
    let data = Datum::sample();

    let mut buf = FakeBuffer::new(14);
    let res = data.write_fixed_all(&mut buf);

    assert!(res.is_err());

    match res.unwrap_err() {
        Error::DataError(_) => panic!("Should have had I/O Error"),
        Error::IoError(e) => {
            assert_eq!(e.to_string(), "Out of space");
            assert_eq!(e.kind(), ErrorKind::WriteZero);
        }
    }

    // Confirm we failed in the unit variant
    let expected = "P5354272   \nU\n";
    assert_eq!(buf.as_string().unwrap(), expected);
}
