use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed]
    field: String,
}

pub fn main() {}
