use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixcol = "path/goes/here"]
    field: String,
}

pub fn main() {}
