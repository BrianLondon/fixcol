use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed(width = 5, align =, default = "none")]
    field: String,
}

pub fn main() {}
