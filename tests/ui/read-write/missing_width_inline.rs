use fixcol::{ReadFixed, WriteFixed};

#[derive(ReadFixed)]
struct Foo(u64);

#[derive(WriteFixed)]
struct Bar(u64);

#[derive(ReadFixed, WriteFixed)]
struct Baz(#[fixcol(width = 20)] u64);

pub fn main() {}
