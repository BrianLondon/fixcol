use std::io::Write;

use crate::error::{DataError, Error};
use crate::format::{Alignment, FieldDescription};
use crate::WriteFixed;

/// A trait that represents the field types that can be encoded to fixed length strings
pub trait FixedSerializer {
    /// Serialize a fixed width representation of the object.
    ///
    /// Uses the provided [`FieldDescription`] to determine how to serialize a fixed
    /// with representation of `self` and writes that representation to the supplie
    /// buffer `buf`.
    fn write_fixed_field<W: Write>(
        &self,
        buf: &mut W,
        desc: &FieldDescription,
    ) -> Result<(), Error>;
}

const SPACES: [u8; 256] = [b' '; 256];

fn write_spaces<W: Write>(buf: &mut W, num: usize) -> Result<(), Error> {
    let mut bytes_to_write: usize = num;

    while bytes_to_write > 256 {
        buf.write(&SPACES)?;
        bytes_to_write -= 256;
    }

    buf.write(&SPACES[..bytes_to_write])?;

    Ok(())
}

impl FixedSerializer for String {
    fn write_fixed_field<W: Write>(
        &self,
        buf: &mut W,
        desc: &FieldDescription,
    ) -> Result<(), Error> {
        // If strict fail on overflow
        if desc.strict && self.len() > desc.len {
            return Err(DataError::new_data_width_error(self.clone(), desc.len, self.len()).into());
        }

        // if strict and full-align fail on too short also
        if desc.strict && desc.alignment == Alignment::Full && self.len() != desc.len {
            return Err(DataError::new_data_width_error(self.clone(), desc.len, self.len()).into());
        }

        // If so we'll need to truncate
        let string_is_too_long = self.len() > desc.len;

        write_spaces(buf, desc.skip)?;

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                if string_is_too_long {
                    buf.write(&self[0..desc.len].as_bytes())?;
                } else {
                    buf.write(&self.as_bytes())?;
                    let spaces_to_pad = desc.len - self.len();
                    write_spaces(buf, spaces_to_pad)?;
                }
            }
            Alignment::Right => {
                if string_is_too_long {
                    let start = self.len() - desc.len;
                    buf.write(&self[start..].as_bytes())?;
                } else {
                    let spaces_to_pad = desc.len - self.len();
                    write_spaces(buf, spaces_to_pad)?;
                    buf.write(&self.as_bytes())?;
                }
            }
        }

        Ok(())
    }
}

macro_rules! fixed_serializer_int_impl {
    ($t:ty) => {
        impl FixedSerializer for $t {
            fn write_fixed_field<W: Write>(
                &self,
                buf: &mut W,
                desc: &FieldDescription,
            ) -> Result<(), Error> {
                let mut s = self.to_string();
                if s.len() > desc.len {
                    if desc.strict {
                        let len = s.len();
                        return Err(DataError::new_data_width_error(s, desc.len, len).into());
                    }
                    // truncate if not strict
                    s = s.as_str()[..desc.len].to_string();
                }

                let padding = desc.len - s.len();

                match desc.alignment {
                    Alignment::Left | Alignment::Full => {
                        write_spaces(buf, desc.skip)?;
                        buf.write(s.as_bytes())?;
                        write_spaces(buf, padding)?;
                    }
                    Alignment::Right => {
                        let skip = padding + desc.skip;
                        write_spaces(buf, skip)?;
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
        desc: &FieldDescription,
    ) -> Result<(), Error> {
        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                write_spaces(buf, desc.skip)?;
                buf.write(s.as_bytes())?;
                write_spaces(buf, padding)?;
            }
            Alignment::Right => {
                let skip = padding + desc.skip;
                write_spaces(buf, skip)?;
                buf.write(s.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl FixedSerializer for f64 {
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
                write_spaces(buf, desc.skip)?;
                buf.write(s.as_bytes())?;
                write_spaces(buf, padding)?;
            }
            Alignment::Right => {
                let skip = padding + desc.skip;
                write_spaces(buf, skip)?;
                buf.write(s.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl<T: WriteFixed> FixedSerializer for T {
    fn write_fixed_field<W: Write>(
        &self,
        buf: &mut W,
        _desc: &FieldDescription,
    ) -> Result<(), Error> {
        self.write_fixed(buf)
    }
}

impl<T: FixedSerializer> FixedSerializer for Option<T> {
    fn write_fixed_field<W: Write>(
        &self,
        buf: &mut W,
        desc: &FieldDescription,
    ) -> Result<(), Error> {
        match self {
            None => String::new().write_fixed_field(buf, desc),
            Some(t) => t.write_fixed_field(buf, desc),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str;

    use regex::Regex;

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
            strict: false,
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
            strict: true,
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
            alignment: Alignment::Full,
            strict: false,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "foo   ");
    }

    #[test]
    fn string_full_strict() {
        // validate "strict" behavior:
        // require written `Full` aligned text columns to be the correct length
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: true,
        };

        let foo = "foo".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_err());
        let e = res.unwrap_err();
        assert_eq!(
            e.to_string(),
            "Error handling data from \"foo\": Expected field to \
            have width 6 but supplied value has width 3.\n"
        );
    }

    #[test]
    fn skip_string_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 6,
            alignment: Alignment::Left,
            strict: true,
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
            strict: true,
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
            strict: true,
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
            strict: false,
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
            strict: false,
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
            strict: false,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " abcd");
    }

    #[test]
    fn overflow_string_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Left,
            strict: true,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Error handling data from \"abcdefg\": Expected field to have width \
            4 but supplied value has width 7.\n"
        );
    }

    #[test]
    fn overflow_string_right() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Right,
            strict: true,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Error handling data from \"abcdefg\": Expected field to have width \
            4 but supplied value has width 7.\n"
        );
    }

    #[test]
    fn overflow_string_full() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Full,
            strict: true,
        };

        let foo = "abcdefg".to_string();

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Error handling data from \"abcdefg\": Expected field to have width \
            4 but supplied value has width 7.\n"
        );
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
            strict: true,
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
            strict: true,
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
            strict: true,
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
            strict: true,
        };

        let foo: i16 = -12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "   -12345");
    }

    #[test]
    fn overflow_u16() {
        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Right,
            strict: true,
        };

        let foo: u16 = 123;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "123");

        let foo: u16 = 12345;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Error handling data from \"12345\": Expected field to have width \
            3 but supplied value has width 5.\n"
        );
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
            strict: true,
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
            strict: true,
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
            strict: true,
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
            strict: true,
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
            strict: true,
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
            strict: true,
        };

        let foo: f32 = 3.141592654;

        let mut v = Vec::new();
        let res = foo.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.1415"); // TODO: should end with 6
    }

    //
    // Long Writes
    ///////////////////////////////////

    #[test]
    fn write_trailing_over_gap() {
        // 200 spaces, 5 chars, 100 spaces
        let desc = FieldDescription {
            skip: 200,
            len: 105,
            alignment: Alignment::Left,
            strict: true,
        };

        let num: u64 = 12345;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {200}12345 {100}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_leading_over_gap() {
        // 300 spaces, 5 chars, 200 spaces
        let desc = FieldDescription {
            skip: 300,
            len: 205,
            alignment: Alignment::Left,
            strict: true,
        };

        let num: u64 = 12345;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {300}12345 {200}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_value_over_gap() {
        // 250 spaces, 10 chars, 300 spaces
        let desc = FieldDescription {
            skip: 250,
            len: 310,
            alignment: Alignment::Left,
            strict: true,
        };

        let num: u64 = 1234567890;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {250}1234567890 {300}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_left_padding_over_gap() {
        // 300 spaces, 295 spaces, 5 chars
        let desc = FieldDescription {
            skip: 300,
            len: 300,
            alignment: Alignment::Right,
            strict: true,
        };

        let num: u64 = 12345;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {595}12345$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_extra_long_spaces_left_align() {
        // 1000 spaces, 5 chars, 995 spaces
        let desc = FieldDescription {
            skip: 1000,
            len: 1000,
            alignment: Alignment::Left,
            strict: true,
        };

        let num: u64 = 12345;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {1000}12345 {995}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_extra_long_spaces_right_align() {
        // 1000 spaces, 995 spaces, 5 chars
        let desc = FieldDescription {
            skip: 1000,
            len: 1000,
            alignment: Alignment::Right,
            strict: true,
        };

        let num: u64 = 12345;

        let mut v: Vec<u8> = Vec::new();
        let res = num.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {1995}12345$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_long_string_left_align() {
        // 1000 spaces, 1000 chars, 1000 spaces
        let desc = FieldDescription {
            skip: 1000,
            len: 2000,
            alignment: Alignment::Left,
            strict: true,
        };

        let s = "abcdefghij".repeat(100);

        let mut v: Vec<u8> = Vec::new();
        let res = s.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {1000}(abcdefghij){100} {1000}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }

    #[test]
    fn write_long_string_right_align() {
        // 1000 spaces, 1000 spaces, 1000 chars
        let desc = FieldDescription {
            skip: 1000,
            len: 2000,
            alignment: Alignment::Right,
            strict: true,
        };

        let s = "abcdefghij".repeat(100);

        let mut v: Vec<u8> = Vec::new();
        let res = s.write_fixed_field(&mut v, &desc);

        assert!(res.is_ok());

        let re = Regex::new(r"^ {2000}(abcdefghij){100}$").unwrap();
        assert!(re.is_match(str::from_utf8(&v).unwrap()));
    }
}
