use crate::format::{Alignment, FieldDescription};

use std::fmt::Display;
use std::io::Write;

/// A trait that represents the field types that can be encoded to fix len strings
pub trait FixedSerializer {
    fn write_fixed<W: Write>(&self, buf: &mut W, desc: &FieldDescription)
        -> Result<(), ()>;
}

const SPACES: [u8; 256] = [b' '; 256];

fn to_unit<T>(_: T) -> () {
    ()
}

impl FixedSerializer for String {
    fn write_fixed<W: Write>(&self, buf: &mut W, desc: &FieldDescription)
            -> Result<(), ()> {
        
        if desc.skip > 256 {
            // TODO: Fix this (also in the FieldDescription docs)
            panic!("Do not currently support skips of more than 256");
        }

        // If so we'll need to truncate
        let string_is_too_long = self.len() > desc.len;

        buf.write(&SPACES[0..desc.skip]).map_err(to_unit)?;

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                if string_is_too_long {
                    buf.write(&self[0..desc.len].as_bytes())
                        .map_err(to_unit)?;
                } else {
                    buf.write(&self.as_bytes()).map_err(to_unit)?;
                    let spaces_to_pad = desc.len - self.len();
                    buf.write(&SPACES[..spaces_to_pad]).map_err(to_unit)?;
                }                
            },
            Alignment::Right => {
                if string_is_too_long {
                    let start = self.len() - desc.len;
                    buf.write(&self[start..].as_bytes()).map_err(to_unit)?;
                } else {
                    let spaces_to_pad = desc.len - self.len();
                    buf.write(&SPACES[..spaces_to_pad]).map_err(to_unit)?;
                    buf.write(&self.as_bytes()).map_err(to_unit)?;
                }
            },
        }

        Ok(())
    }
}

// Dummy trait to limit application of generic FixedSerializer
trait IntegerLike {}

impl IntegerLike for u8 {}
impl IntegerLike for u16 {}
impl IntegerLike for u32 {}
impl IntegerLike for u64 {}

impl IntegerLike for i8 {}
impl IntegerLike for i16 {}
impl IntegerLike for i32 {}
impl IntegerLike for i64 {}

// TODO: make this handle overflows
impl<D: Display + IntegerLike> FixedSerializer for D {
    fn write_fixed<W: Write>(&self, buf: &mut W, desc: &FieldDescription)
            -> Result<(), ()> 
    {
        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                buf.write(&SPACES[..desc.skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
                buf.write(&SPACES[..padding]).map_err(to_unit)?;
            },
            Alignment::Right => {
                let skip = padding + desc.skip;
                buf.write(&SPACES[..skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
            },
        }

        Ok(())
    }
}

// TODO: These are likely completely broken and need to support fmt options
impl FixedSerializer for f32 {
    fn write_fixed<W: Write>(&self, buf: &mut W, desc: &FieldDescription)
            -> Result<(), ()> 
    {
        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                buf.write(&SPACES[..desc.skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
                buf.write(&SPACES[..padding]).map_err(to_unit)?;
            },
            Alignment::Right => {
                let skip = padding + desc.skip;
                buf.write(&SPACES[..skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
            },
        }

        Ok(())
    }
}

impl FixedSerializer for f64 {
    fn write_fixed<W: Write>(&self, buf: &mut W, desc: &FieldDescription)
            -> Result<(), ()> 
    {
        let mut s = self.to_string();
        if s.len() > desc.len {
            s = s.as_str()[..desc.len].to_string();
        }

        let padding = desc.len - s.len();

        match desc.alignment {
            Alignment::Left | Alignment::Full => {
                buf.write(&SPACES[..desc.skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
                buf.write(&SPACES[..padding]).map_err(to_unit)?;
            },
            Alignment::Right => {
                let skip = padding + desc.skip;
                buf.write(&SPACES[..skip]).map_err(to_unit)?;
                buf.write(s.as_bytes()).map_err(to_unit)?;
            },
        }

        Ok(())
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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

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
        let res = foo.write_fixed(&mut v, &desc);

        assert!(res.is_ok());
        assert_eq!(to_str(v), " 3.1415"); // TODO: should end with 6
    }
}