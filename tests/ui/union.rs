use fixcol_derive::{ReadFixed, WriteFixed};

#[derive(ReadFixed, WriteFixed)]
#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

pub fn main() {}
