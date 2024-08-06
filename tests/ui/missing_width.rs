use fixed_derive::{ReadFixed};

#[derive(ReadFixed)]
struct Point {
    #[fixed(width = 5)]
    x: u16,
    y: u16,
}

pub fn main() {}
