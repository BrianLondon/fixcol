use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixcol]
    field: String,
}

pub fn main() {}
