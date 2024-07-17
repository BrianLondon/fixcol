mod fixed;

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
    fn test_custom() {
        struct Foo;

        impl WriteFixed for Foo {
            fn write_fixed(&self, write: &mut dyn std::io::Write) -> Result<(), ()> {
                write.write("Foo".as_bytes()).map(|_| ()).map_err(|_| ())
            }
        }

        let foo = Foo;

        let mut v = Vec::new();
        let res = foo.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "Foo");
    }
}
