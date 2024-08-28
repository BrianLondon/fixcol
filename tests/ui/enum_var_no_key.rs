use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixcol(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixcol(key_width = 3)]
enum Thing1 {
    #[fixcol(key = "one")]
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
#[fixcol(key_width = 3)]
enum Thing2 {
    Thing1(#[fixcol(width = 5)] u16),
    #[fixcol(embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}
