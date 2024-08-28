use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixcol(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixcol(ignore_others = 1)]
enum Thing {
    #[fixcol(key = "one")]
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(key = "two", embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}
