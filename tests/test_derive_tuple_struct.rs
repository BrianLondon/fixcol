extern crate fixcol;

use fixcol::{ReadFixed, WriteFixed};

#[derive(Debug, Eq, PartialEq, ReadFixed, WriteFixed)]
struct Color(
    #[fixcol(width = 3, align = "right")] u8,
    #[fixcol(skip = 1, width = 3, align = "right")] u8,
    #[fixcol(skip = 1, width = 3, align = "right")] u8,
);

#[test]
fn derive_read() {
    let mut buf = "255 255  64".as_bytes();

    let color = Color::read_fixed(&mut buf).unwrap();
    assert_eq!(color, Color(255, 255, 64));
}

#[test]
fn derive_write() {
    let mut buf = Vec::new();
    let res = Color(0, 128, 42).write_fixed(&mut buf);

    assert!(res.is_ok());

    let s = std::str::from_utf8(buf.as_slice()).unwrap();
    assert_eq!(s, "  0 128  42");
}
