use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixcol(width = 5 align = right)]
    field: String,
}

pub fn main() {}
