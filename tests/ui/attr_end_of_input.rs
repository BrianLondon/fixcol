use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed(width = 5, align =)]
    field: String,
}

pub fn main() {}