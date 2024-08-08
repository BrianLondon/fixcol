use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixed(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixed(ignore_others = true)]
enum Thing1 {
    #[fixed(key = "one")]
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(key = "two", embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
enum Thing2 {
    #[fixed(key = "one")]
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(key = "two", embed = true)]
    Thing2(MyStruct),
}

#[derive(ReadFixed)]
#[fixed(key_width = three)]
enum Thing3 {
    #[fixed(key = "one")]
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(key = "two", embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}
