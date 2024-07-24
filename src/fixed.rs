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

    /// Reads an instance of the object fom a `&str`
    /// 
    /// Deserializes a single item of the type from a fixed width representation
    /// of the object stored in a `&str`.
    /// 
    /// # Examples
    /// 
    /// We can parse directly from `str` literals
    /// ```
    /// # use fixed_derive::ReadFixed;
    /// # use fixed::FixedDeserializer;
    /// # use fixed::FieldDescription;
    /// #[derive(ReadFixed)]
    /// struct Point {
    ///     #[fixed(width=3, align="right")]
    ///     x: u8,
    ///     #[fixed(width=3, align="right")]
    ///     y: u8,
    /// }
    /// 
    /// # use fixed::ReadFixed;
    /// let point = Point::read_fixed_str(" 42  7").unwrap();
    /// assert_eq!(point.x, 42);
    /// assert_eq!(point.y, 7)
    /// ```
    /// 
    /// It can also be useful to pull directly from slices.
    /// ```
    /// # use fixed_derive::ReadFixed;
    /// # use fixed::FixedDeserializer;
    /// # use fixed::FieldDescription;
    /// # #[derive(ReadFixed)]
    /// # struct Point {
    /// #     #[fixed(width=3)]
    /// #     x: u8,
    /// #     #[fixed(width=3)]
    /// #     y: u8,
    /// # }   
    ///
    /// # use fixed::ReadFixed;
    /// let s = ">>12361 <<";
    /// let point = Point::read_fixed_str(&s[2..8]).unwrap();
    /// 
    /// assert_eq!(point.x, 123);
    /// assert_eq!(point.y, 61);
    /// ```
    fn read_fixed_str(s: &str) -> Result<Self, ()> 
        where Self: Sized
    {
        let mut bytes = s.as_bytes();
        Self::read_fixed(&mut bytes)
    }

    /// Reads an instance of the object fom a [`String`]
    fn read_fixed_string(s: String) -> Result<Self, ()> 
        where Self: Sized
    {
        let mut bytes = s.as_bytes();
        Self::read_fixed(&mut bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo {
        foo: String,
    }

    impl ReadFixed for Foo {
        fn read_fixed<R>(buf: &mut R) -> Result<Self, ()>
            where Self: Sized, R: Read {
            
            let mut s: String = String::new();
            buf.read_to_string(&mut s).map_err(|_| ())?;

            Ok(Self { foo: s })
        }
    }

    #[test]
    fn read_fixed_str() {
        let foo = Foo::read_fixed_str("bar");
        assert_eq!(foo.unwrap(), Foo{ foo: "bar".to_string()});
    }

    #[test]
    fn read_fixed_string() {
        let s: String = "bar".to_string();
        let foo = Foo::read_fixed_string(s);
        assert_eq!(foo.unwrap(), Foo{ foo: "bar".to_string()});
    }
}