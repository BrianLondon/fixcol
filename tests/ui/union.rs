use fixed_derive::{ReadFixed};

#[derive(ReadFixed)]
#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

pub fn main() {}
