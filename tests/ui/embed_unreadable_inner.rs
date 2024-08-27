use fixcol_derive::{ReadFixed, WriteFixed};

struct Point {
    point_x: u16,
    point_y: u16,
}

#[derive(ReadFixed, WriteFixed)]
#[fixed(key_width = 1)]
enum Alg {
    #[fixed(key = "N")]
    Num(#[fixed(width = 5)] u16),
    #[fixed(key = "P", embed = true)]
    Point(Point),
    #[fixed(key = "Z")]
    Zero,
}

pub fn main() {}
