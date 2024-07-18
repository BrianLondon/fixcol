use std::io;


/// Trait for writing to fixed width (column based) serialization
pub trait WriteFixed {
    /// Writes the object into the supplied buffer
    fn write_fixed(&self, buf: &mut dyn io::Write) -> Result<(), ()>;
}

/// Trait for reading from fixed width (column based) serializaiton
pub trait ReadFixed {
    /// Reads an instance of the object from the supplied buffer
    fn read_fixed(buf: &mut impl io::Read) -> Result<Self, ()>
        where Self: Sized;
}
