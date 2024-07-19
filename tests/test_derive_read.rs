extern crate fixed;
extern crate fixed_derive;

use fixed_derive::ReadFixed;


#[derive(Debug, ReadFixed, Eq, PartialEq)]
struct Point {
    x: u64,
    y: u64,
}

#[test]
fn derive_read_struct() {
    use fixed::ReadFixed;  // TODO: this double import is really ugly

    let mut buf = "42        3         ".as_bytes();
    let point = Point::read_fixed(&mut buf).unwrap();
    assert_eq!(point, Point { x: 42, y: 3 });    
}

