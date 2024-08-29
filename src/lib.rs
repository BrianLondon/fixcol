//! A crate used for *fixed* width *column* serialization and deserialization
//! 
//! Fixcol provides a derive based deserialization framework for parsing text files
//! with fixed column width serialization formats. While file formats using character
//! delimeters such as CSV or JSON are more common today, fixed column file formats
//! are more naturally human readable and many older data sets, especially public
//! domain data sets continue to use them.
#![feature(doc_auto_cfg)]

pub mod error;
mod fixcol;
mod format;
mod parse;

#[cfg(feature = "experimental-write")]
mod write;

extern crate fixcol_derive;

pub use fixcol::{Iter, ReadFixed};
pub use format::{Alignment, FieldDescription};
pub use parse::FixedDeserializer;

#[cfg(feature = "experimental-write")]
pub use fixcol::{WriteFixed, WriteFixedAll};
#[cfg(feature = "experimental-write")]
pub use write::FixedSerializer;

pub use fixcol_derive::ReadFixed;
#[cfg(feature = "experimental-write")]
pub use fixcol_derive::WriteFixed;

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
    #[cfg(feature = "experimental-write")]
    fn write_custom_basic() {
        struct Foo;

        impl WriteFixed for Foo {
            fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
                let _ = buf.write("Foo".as_bytes())?;
                Ok(())
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

    #[cfg(feature = "experimental-write")]
    impl WriteFixed for NumWord {
        fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
            let _ = buf.write_fmt(format_args!("{:<10}{:>3}", self.name, self.value))?;
            Ok(())
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

    #[cfg(feature = "experimental-write")]
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
