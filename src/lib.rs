//! A crate used for fixed width column serialization and deserialization
pub mod error;
mod fixed;
mod format;
mod parse;
mod write;

extern crate fixed_derive;

pub use fixed::{Iter, ReadFixed, WriteFixed, WriteFixedAll};
pub use format::{Alignment, FieldDescription};
pub use parse::FixedDeserializer;
pub use write::FixedSerializer;

// TODO: should we support custom deserialization functions on individual columns?

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use error::Error;

    use super::*;

    fn to_str(inp: Vec<u8>) -> String {
        use std::str;
        str::from_utf8(inp.as_slice()).unwrap().to_string()
    }

    #[test]
    fn test_helper() {
        let buf: [u8; 13] = [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33];

        let mut v = Vec::new();
        let _ = v.write(&buf);

        assert_eq!(to_str(v), "Hello, World!");
    }

    #[test]
    fn write_custom_basic() {
        struct Foo;

        impl WriteFixed for Foo {
            fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
                buf.write("Foo".as_bytes()).map_err(|e| Error::from(e))
            }
        }

        let foo = Foo;

        let mut v = Vec::new();
        let res = foo.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "Foo");
    }

    // Name is left aligned, ten characters
    // value is right aligned three characters
    #[derive(Debug, PartialEq, Eq)]
    struct NumWord {
        name: String,
        value: u8,
    }

    impl WriteFixed for NumWord {
        fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
            let _ = buf.write_fmt(format_args!("{:<10}{:>3}", self.name, self.value))?;
            Ok(0)
        }
    }

    impl ReadFixed for NumWord {
        fn read_fixed<R: Read>(buf: &mut R) -> Result<Self, Error>
        where
            Self: Sized,
        {
            let mut s = String::new();
            let _ = buf.read_to_string(&mut s);

            let name = s[0..10].trim_end().to_string();
            let num = s[10..].to_string();
            let value = num.trim_start().parse::<u8>().unwrap();

            Ok(NumWord { name, value })
        }
    }

    #[test]
    fn custom_struct_write() {
        let three = NumWord { name: "three".to_string(), value: 3 };

        let mut v = Vec::new();
        let res = three.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "three       3");
    }

    #[test]
    fn custom_struct_read() {
        let three = NumWord { name: "three".to_string(), value: 3 };

        let mut buf = "three       3".as_bytes();
        let decoded = NumWord::read_fixed(&mut buf).unwrap();

        assert_eq!(decoded, three);
    }
}
