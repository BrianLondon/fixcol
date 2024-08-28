use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct Thing {
    #[fixcol(width = 5, align = "right")]
    field_a: String,
    #[fixcol(width = 5, thing = true)]
    field_b: String,
    #[fixcol(width = 5)]
    field_c: String,
}

pub fn main() {}
