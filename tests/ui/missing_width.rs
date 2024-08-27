use fixcol_derive::{ReadFixed};

#[derive(ReadFixed)]
struct Item {
    #[fixed(width = 5)]
    id: u64,
    name: String,
}

pub fn main() {}
