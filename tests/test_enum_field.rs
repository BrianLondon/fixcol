extern crate fixed;
use fixed::{ReadFixed, WriteFixed};

#[derive(Debug, Eq, PartialEq, ReadFixed, WriteFixed)]
#[fixed(key_width = 1)]
enum Color {
    #[fixed(key = "R")]
    Red,
    #[fixed(key = "G")]
    Green,
    #[fixed(key = "B")]
    Blue,
}

#[derive(Debug, ReadFixed, WriteFixed, Eq, PartialEq)]
struct Light {
    #[fixed(width = 8)]
    name: String,
    #[fixed(width = 1)] // TODO: this is redundant -- find a way to inherit
    color: Color,
    #[fixed(width = 3, align = "right")]
    pos_x: u8,
    #[fixed(width = 3, align = "right")]
    pos_y: u8,
}

#[test]
fn derive_read() {
    let expected = Light {
        name: String::from("RtBl2"),
        color: Color::Blue,
        pos_x: 174,
        pos_y: 34,
    };

    let mut buf = "RtBl2   B174 34".as_bytes();
    let actual = Light::read_fixed(&mut buf).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn derive_write() {
    use std::str;

    let expected = "RtBl2   B174 34";

    let light = Light {
        name: String::from("RtBl2"),
        color: Color::Blue,
        pos_x: 174,
        pos_y: 34,
    };

    let mut buf: Vec<u8> = Vec::new();
    let res = light.write_fixed(&mut buf);

    assert!(res.is_ok());
    let actual = str::from_utf8(buf.as_slice()).unwrap();

    assert_eq!(actual, expected);
}
