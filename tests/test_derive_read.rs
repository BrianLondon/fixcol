extern crate fixed;
extern crate fixed_derive;

use fixed_derive::{ReadFixed, WriteFixed};


#[derive(Debug, ReadFixed, WriteFixed, Eq, PartialEq)]
struct Point {
    #[fixed(width=10, align=left)]
    x: u64,
    /// The y coordinate
    #[fixed(width=10, strict=true, align="right")]
    #[allow(non_camel_case_types)]
    y: u64,
}

#[test]
fn derive_read_struct() {
    use fixed::ReadFixed;  // TODO: this double import is really ugly

    let mut buf = "42        3         ".as_bytes();
    let point = Point::read_fixed(&mut buf).unwrap();
    assert_eq!(point, Point { x: 42, y: 3 });    
}

