use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixed(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixed(key_width = 3)]
enum Thing1 {
    #[fixed(key = "one")]
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
#[fixed(key_width = 3)]
enum Thing2 {
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}
