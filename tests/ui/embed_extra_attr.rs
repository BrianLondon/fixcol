use fixed_derive::{ReadFixed, WriteFixed};

#[derive(ReadFixed)]
struct Point {
    #[fixed(width = 5)]
    point_x: u16,
    #[fixed(width = 5)]
    point_y: u16,
}

#[derive(ReadFixed, WriteFixed)]
#[fixed(key_width = 1)]
enum Alg {
    #[fixed(key = "N")]
    Num(#[fixed(width = 5)] u16),
    #[fixed(key = "P", embed = true)]
    Point(#[fixed(width = 5)] Point),
    #[fixed(key = "Z")]
    Zero,
}

pub fn main() {}
