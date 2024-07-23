use std::io::{Read, Write};

/// Trait for writing to fixed width (column based) serialization
pub trait WriteFixed {
    /// Writes the object into the supplied buffer
    fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), ()>;
}

/// Trait for reading from fixed width (column based) serializaiton
pub trait ReadFixed {
    /// Reads an instance of the object from the supplied buffer
    fn read_fixed<R>(buf: &mut R) -> Result<Self, ()>
        where Self: Sized, R: Read;
}
