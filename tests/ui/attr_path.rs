use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed]
    field: String,
}

pub fn main() {}
