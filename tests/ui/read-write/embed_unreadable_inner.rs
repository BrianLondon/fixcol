use fixcol::ReadFixed;
#[cfg(feature = "experimental-write")]
use fixcol::WriteFixed;

struct Point {
    point_x: u16,
    point_y: u16,
}

#[derive(ReadFixed)]
#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
#[fixcol(key_width = 1)]
enum Alg {
    #[fixcol(key = "N")]
    Num(#[fixcol(width = 5)] u16),
    #[fixcol(key = "P", embed = true)]
    Point(Point),
    #[fixcol(key = "Z")]
    Zero,
}

pub fn main() {}
