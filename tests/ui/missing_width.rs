use fixcol_derive::{ReadFixed};

#[derive(ReadFixed)]
struct Item {
    #[fixcol(width = 5)]
    id: u64,
    name: String,
}

pub fn main() {}
