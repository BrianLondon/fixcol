use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixed(width = 5, align = "right")]
    field_a: String,
    #[fixed(width = 5, thing = true)]
    field_b: String,
    #[fixed(width = 5)]
    field_c: String,
}

pub fn main() {}
