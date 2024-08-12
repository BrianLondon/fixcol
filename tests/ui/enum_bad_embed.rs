use fixed_derive::ReadFixed;

#[derive(ReadFixed)]
struct MyStruct {
    #[fixed(width = 5)]
    my_int: u16
}

#[derive(ReadFixed)]
#[fixed(key_width = 3)]
enum Thing {
    #[fixed(key = "one", embed = true)]
    Thing1(#[fixed(width = 5)] u16),
    #[fixed(key = "two", embed = true)]
    Thing2(MyStruct),
}

pub fn main() {}