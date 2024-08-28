use fixcol::ReadFixed;

#[derive(ReadFixed)]
struct Foo(u64);

#[derive(ReadFixed)]
struct Baz(#[fixcol(width = 20)] u64);

pub fn main() {}
