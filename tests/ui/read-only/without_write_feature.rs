use fixcol::{ReadFixed, WriteFixed};

#[derive(ReadFixed, WriteFixed)]
struct Point {
    #[fixcol(width = 5)]
    point_x: u16,
    #[fixcol(width = 5)]
    point_y: u16,
}

fn main() {}
