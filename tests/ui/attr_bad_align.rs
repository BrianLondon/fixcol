use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed(width = 5, align = "backwards")]
    field: String,
}

pub fn main() {}
