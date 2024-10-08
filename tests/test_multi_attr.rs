extern crate fixcol;

use fixcol::ReadFixed;
#[cfg(feature = "experimental-write")]
use fixcol::WriteFixed;

#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
#[derive(Debug, ReadFixed, Eq, PartialEq)]
struct Point {
    /// The x coordinate
    #[fixcol(width = 5, align = "right")]
    x: u16,

    /// The y coordinate
    #[fixcol(width = 5)]
    #[fixcol(align = "right")]
    y: u16,
}

#[test]
fn derive_read_struct() {
    let mut buf = "   42  212".as_bytes();
    let point = Point::read_fixed(&mut buf).unwrap();
    assert_eq!(point, Point { x: 42, y: 212 });
}

#[cfg(feature = "experimental-write")]
#[test]
fn derive_write_struct() {
    let point = Point { x: 42, y: 212 };

    let mut v = Vec::new();
    let res = point.write_fixed(&mut v);

    assert!(res.is_ok());
    assert_eq!(
        std::str::from_utf8(v.as_slice()).unwrap(),
        std::str::from_utf8("   42  212".as_bytes()).unwrap()
    );
}
