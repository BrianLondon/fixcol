use std::io;

pub trait WriteFixed {
    fn write_fixed(&self, write: &mut dyn io::Write) -> Result<(), ()>;
}


pub trait ReadFixed {
    fn read_fixed(read: &mut impl io::Read) -> Result<Self, ()>
        where Self: Sized;
}
