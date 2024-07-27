extern crate fixed;
extern crate fixed_derive;

use fixed_derive::{ReadFixed, WriteFixed};

#[derive(Debug, ReadFixed, WriteFixed, Eq, PartialEq)]
struct Point {
    /// The x coordinate
    #[fixed(width=10, align=left)]
    x: u64,
    /// The y coordinate
    // #[fixed(width=10, strict=true, align="right")] TODO: add strict back
    #[fixed(skip=1, width=9, align=right)]
    y: u64,
}

#[test]
fn derive_read_struct() {
    use fixed::ReadFixed; // TODO: this double import is really ugly

    let mut buf = "42                 3".as_bytes();
    let point = Point::read_fixed(&mut buf).unwrap();
    assert_eq!(point, Point { x: 42, y: 3 });
}

#[test]
fn derive_write_struct() {
    use fixed::WriteFixed;

    let point = Point { x: 42, y: 3 };

    let mut v = Vec::new();
    let res = point.write_fixed(&mut v);

    assert!(res.is_ok());
    assert_eq!(
        std::str::from_utf8(v.as_slice()).unwrap(),
        std::str::from_utf8("42                 3".as_bytes()).unwrap()
    );
}
