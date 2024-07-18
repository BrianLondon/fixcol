//! A crate used for fixed width column serialization and deserialization
mod fixed;

extern crate fixed_derive;

pub use fixed::{ReadFixed, WriteFixed};

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn to_str(inp: Vec<u8>) -> String {
        use std::str;
        str::from_utf8(inp.as_slice()).unwrap().to_string()
    }

    #[test]
    fn test_helper() {
        let buf: [u8; 13] =
            [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33];

        let mut v = Vec::new();
        let _ = v.write(&buf);

        assert_eq!(to_str(v), "Hello, World!");
    }

    #[test]
    fn test_write_custom_basic() {
        struct Foo;

        impl WriteFixed for Foo {
            fn write_fixed(&self, buf: &mut dyn std::io::Write) -> Result<(), ()> {
                buf.write("Foo".as_bytes()).map(|_| ()).map_err(|_| ())
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
        fn write_fixed(&self, buf: &mut dyn std::io::Write) -> Result<(), ()> {
            let _ = buf.write_fmt(format_args!("{:<10}{:>3}", self.name, self.value));
            Ok(())
        }
    }

    impl ReadFixed for NumWord {
        fn read_fixed(read: &mut impl std::io::Read) -> Result<Self, ()>
                where Self: Sized {
            let mut s = String::new();
            let _ = read.read_to_string(&mut s);

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

    // TODO: Delete me (this is not correct behavior)
    #[test]
    fn derive_dummy() {
        use fixed_derive::WriteFixed;
        
        #[derive(WriteFixed)]
        struct Point {
            x: u64,
            y: u64,
        }

        let point = Point { x: 42, y: 3 };

        let mut v = Vec::new();
        let res = point.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "42        3         ");
    }
}
