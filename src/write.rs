use std::io::Write;

use crate::error::Error;
use crate::format::{Alignment, FieldDescription};
use crate::WriteFixed;

/// A trait that represents the field types that can be encoded to fix len strings
pub trait FixedSerializer {
    /// Serialize a fixed with representation of the object.
    ///
    /// Uses the provided [`FieldDescription`] to determine how to serialize a fixed
    /// with representation of `self` and writes that representation to the supplie
    /// buffer `buf`.
    fn write_fixed_field<W: Write>(&self, buf: &mut W, desc: &FieldDescription) -> Result<(), Error>;
}

const SPACES: [u8; 256] = [b' '; 256];

fn to_unit<T>(_: T) -> () {
    ()
}

impl FixedSerializer for String {
    fn write_fixed_field<W: Write>(&self, buf: &mut W, desc: &FieldDescription) -> Result<(), Error> {
        if desc.skip > 256 {
            // TODO: Fix this (also in the FieldDescription docs)
            panic!("Do not currently support skips of more than 256");
        }

        // If so we'll need to truncate
        let string_is_too_long = self.len() > desc.len;

        buf.write(&SPACES[0..desc.skip])?;

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                if string_is_too_long {
                    buf.write(&self[0..desc.len].as_bytes())?;
                } else {
                    buf.write(&self.as_bytes())?;
                    let spaces_to_pad = desc.len - self.len();
                    buf.write(&SPACES[..spaces_to_pad])?;
                }
            }
            Alignment::Right => {
                if string_is_too_long {
                    let start = self.len() - desc.len;
                    buf.write(&self[start..].as_bytes())?;
                } else {
                    let spaces_to_pad = desc.len - self.len();
                    buf.write(&SPACES[..spaces_to_pad])?;
                    buf.write(&self.as_bytes())?;
                }
            }
        }

        Ok(())
    }
}

macro_rules! fixed_serializer_int_impl {
    ($t:ty) => {
        // TODO: make this handle overflows
        impl FixedSerializer for $t {
            fn write_fixed_field<W: Write>(
                &self,
                buf: &mut W,
                desc: &FieldDescription,
            ) -> Result<(), Error> {

                let mut s = self.to_string();
                if s.len() > desc.len {
                    s = s.as_str()[..desc.len].to_string();
                }

                let padding = desc.len - s.len();

                match desc.alignment {
                    Alignment::Left | Alignment::Full => {
                        buf.write(&SPACES[..desc.skip])?;
                        buf.write(s.as_bytes())?;
                        buf.write(&SPACES[..padding])?;
                    }
                    Alignment::Right => {
                        let skip = padding + desc.skip;
                        buf.write(&SPACES[..skip])?;
                        buf.write(s.as_bytes())?;
                    }
                }

                Ok(())
            }
        }
    };
}

fixed_serializer_int_impl!(u8);
fixed_serializer_int_impl!(u16);
fixed_serializer_int_impl!(u32);
fixed_serializer_int_impl!(u64);

fixed_serializer_int_impl!(i8);
fixed_serializer_int_impl!(i16);
fixed_serializer_int_impl!(i32);
fixed_serializer_int_impl!(i64);

fixed_serializer_int_impl!(usize);
fixed_serializer_int_impl!(isize);

// TODO: These are likely completely broken and need to support fmt options
impl FixedSerializer for f32 {
    fn write_fixed_field<W: Write>(
        &self,
        buf: &mut W,
        desc: &FieldDescription
    ) -> Result<(), Error> {
        let mut bytes_written: usize = 0;

        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                bytes_written += buf.write(&SPACES[..desc.skip])?;
                bytes_written += buf.write(s.as_bytes())?;
                bytes_written += buf.write(&SPACES[..padding])?;
            }
            Alignment::Right => {
                let skip = padding + desc.skip;
                bytes_written += buf.write(&SPACES[..skip])?;
                bytes_written += buf.write(s.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl FixedSerializer for f64 {
    fn write_fixed_field<W: Write>(&self, buf: &mut W, desc: &FieldDescription) -> Result<(), Error> {
        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                buf.write(&SPACES[..desc.skip])?;
                buf.write(s.as_bytes())?;
                buf.write(&SPACES[..padding])?;
            }
            Alignment::Right => {
                let skip = padding + desc.skip;
                buf.write(&SPACES[..skip])?;
                buf.write(s.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl<T: WriteFixed> FixedSerializer for T {
    fn write_fixed_field<W: Write>(&self, buf: &mut W, _desc: &FieldDescription) -> Result<(), Error> {
        self.write_fixed(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_str(inp: Vec<u8>) -> String {
        use std::str;
        str::from_utf8(inp.as_slice()).unwrap().to_string()
    }

    //
    // String writes
    /////////////////////////////////

    #[test]
    fn pad_string_left() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "foo   ");
    }

    #[test]
    fn pad_string_right() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "   foo");
    }

    #[test]
    fn pad_string_full() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "foo   ");
    }

    #[test]
    fn skip_string_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " foo   ");
    }

    #[test]
    fn skip_string_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Right,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "    foo");
    }

    #[test]
    fn skip_string_full() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " foo   ");
    }

    #[test]
    fn truncate_string_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Left,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " abcd");
    }

    #[test]
    fn truncate_string_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Right,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " defg");
    }

    #[test]
    fn truncate_string_full() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Left,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " abcd");
    }

    //
    // Integer writes
    ////////////////////////////////////////////

    #[test]
    fn write_u16_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo: u16 = 12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 12345 ");
    }

    #[test]
    fn write_u16_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Right,
        };

        let foo: u16 = 12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "  12345");
    }

    #[test]
    fn write_i16_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 8,
            alignment: Alignment::Left,
        };

        let foo: i16 = -12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " -12345  ");
    }

    #[test]
    fn write_i16_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 8,
            alignment: Alignment::Right,
        };

        let foo: i16 = -12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "   -12345");
    }

    //
    // Floating point checks
    ///////////////////////////////

    #[test]
    fn write_f32_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo: f32 = 3.14;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.14  ");
    }

    #[test]
    fn write_f32_left_trucnate() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
        };

        let foo: f32 = 3.141592654;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.1415"); // TODO: should end with 6
    }

    #[test]
    fn write_f32_full() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Full,
        };

        let foo: f32 = 3.14;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.14  ");
    }

    #[test]
    fn write_f32_full_trucnate() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Full,
        };

        let foo: f32 = 3.141592654;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.1415"); // TODO: should end with 6
    }

    #[test]
    fn write_f32_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Right,
        };

        let foo: f32 = 3.14;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "   3.14");
    }

    #[test]
    fn write_f32_right_trucnate() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Right,
        };

        let foo: f32 = 3.141592654;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.1415"); // TODO: should end with 6
    }
}
