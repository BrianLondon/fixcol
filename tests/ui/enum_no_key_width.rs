use fixcol_derive::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixcol(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixcol(ignore_others = true)]
enum Thing1 {
    #[fixcol(key = "one")]
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(key = "two", embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
enum Thing2 {
    #[fixcol(key = "one")]
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(key = "two", embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
#[fixcol(key_width = three)]
enum Thing3 {
    #[fixcol(key = "one")]
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(key = "two", embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}
