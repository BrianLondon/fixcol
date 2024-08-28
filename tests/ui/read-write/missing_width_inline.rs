use fixcol::ReadFixed;
#[cfg(feature = "experimental-write")]
use fixcol::WriteFixed;

#[derive(ReadFixed)]
struct Foo(u64);

#[cfg(feature = "experimental-write")]
#[derive(WriteFixed)]
struct Bar(u64);

#[derive(ReadFixed)]
#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
struct Baz(#[fixcol(width = 20)] u64);

pub fn main() {}
