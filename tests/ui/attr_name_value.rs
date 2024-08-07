use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed = "path/goes/here"]
    field: String,
}

pub fn main() {}
