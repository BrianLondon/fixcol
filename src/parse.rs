use crate::format::{Alignment, FieldDescription};

use std::str::FromStr;


/// A trait the represents field types that can be decoded from fixed len strings
/// 
/// Implementations are provided for `&str` that are used internally in the macro
/// generated code to derive [`ReadFixed`]. It is unlikely end users will need to
/// implement this trait for other types.
/// 
/// It may be useful to implement this for other `T` on &str if you would like
/// to directly deserialize other primitives.
/// 
/// # Example
/// 
/// ```
/// # use fixed_derive::ReadFixed;
/// # use fixed::FixedDeserializer;
/// # use fixed::FieldDescription;
/// #[derive(PartialEq, Eq, Debug)]
/// enum EyeColor {
///     Blue,
///     Brown,
///     Green,
/// }
/// 
/// impl FixedDeserializer<EyeColor> for &str {
///     fn parse_with(&self, desc: &FieldDescription) -> Result<EyeColor, ()> {
///         match *self {
///             "Bl" => Ok(EyeColor::Blue),
///             "Br" => Ok(EyeColor::Brown),
///             "Gr" => Ok(EyeColor::Green),
///             _ => Err(())
///         }
///     }
/// }
/// 
/// #[derive(ReadFixed)]
/// struct Person {
///     #[fixed(width=10)]
///     pub name: String,
///     #[fixed(width=3, align=right)]
///     pub age: u8,
///     #[fixed(width=2)]
///     pub eye_color: EyeColor,
/// }
/// 
/// # use fixed::ReadFixed;
/// let person = Person::read_fixed_str("Harold     42Gr").unwrap();
/// assert_eq!(person.eye_color, EyeColor::Green);
/// ```
pub trait FixedDeserializer<T : Sized> {
    /// Read an object of type `T` from the current object.
    /// 
    /// Uses the provided [`FieldDescription`] to determine how to parse a data field
    /// from a fixed width representation.
    fn parse_with(&self, desc: &FieldDescription) -> Result<T, ()>;
}


fn extract_trimmed<'a, 'b>(src: &'a str, desc: &'b FieldDescription) -> &'a str {
    let slice = &src[desc.skip..desc.skip+desc.len];
        
    match desc.alignment {
        Alignment::Left => slice.trim_end(),
        Alignment::Right => slice.trim_start(),
        Alignment::Full => slice,
    }
}


// Dummy trait to limit the application of the generic FixedDeseralizable
trait Numeric {}

impl Numeric for u8 {}
impl Numeric for u16 {}
impl Numeric for u32 {}
impl Numeric for u64 {}

impl Numeric for i8 {}
impl Numeric for i16 {}
impl Numeric for i32 {}
impl Numeric for i64 {}

impl Numeric for f32 {}
impl Numeric for f64 {}


impl<T: FromStr + Numeric> FixedDeserializer<T> for &str {
    fn parse_with(&self, desc: &FieldDescription) -> Result<T, ()> {
        let trimmed = extract_trimmed(self, desc);
        trimmed.parse::<T>().map_err(|_| ())
    }
}


impl FixedDeserializer<String> for &str {
    fn parse_with(&self, desc: &FieldDescription) -> Result<String, ()> {
        let trimmed = extract_trimmed(self, desc);
        Ok(trimmed.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_string_left() {
        let desc = FieldDescription{ skip: 0, len: 3, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_pad() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_truncate() {
        let desc = FieldDescription{ skip: 0, len: 2, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = "a bc  ".parse_with(&desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_leading_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = " abc  ".parse_with(&desc).unwrap();
        let expected = " abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_exact() {
        let desc = FieldDescription{ skip: 0, len: 3, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip_into() {
        let desc = FieldDescription{ skip: 4, len: 2, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_truncate() {
        let desc = FieldDescription{ skip: 1, len: 4, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = "  a bc".parse_with(&desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_trailing_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = " abc  ".parse_with(&desc).unwrap();
        let expected = "abc  ".to_string();
        assert_eq!(actual, expected)
    }


    #[test]
    fn extract_string_full() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "abcdef".parse_with(&desc).unwrap();
        let expected = "abcdef".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_slice() {
        let desc = FieldDescription{ skip: 1, len: 3, alignment: Alignment::Full};
        let actual: String = "abcdef".parse_with(&desc).unwrap();
        let expected = "bcd".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_left() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "abc   ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_right() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "   abc".parse_with(&desc).unwrap();
        let expected = "   abc".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "bc   ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_truncate() {
        let desc = FieldDescription{ skip: 0, len: 4, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(&desc).unwrap();
        let expected = "abc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = " a bc ".parse_with(&desc).unwrap();
        let expected = " a bc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_trimmed_ws() {
        let desc = FieldDescription{ skip: 1, len: 3, alignment: Alignment::Full};
        let actual: String = " ab c ".parse_with(&desc).unwrap();
        let expected = "ab ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_tight_wc() {
        let desc = FieldDescription{ skip: 1, len: 4, alignment: Alignment::Full};
        let actual: String = " ab c ".parse_with(&desc).unwrap();
        let expected = "ab c".to_string();
        assert_eq!(actual, expected);
    }

    /* TODO: This is the behavior for a future non-strict implementation
    #[test]
    fn extract_f32_padding() {
        let descs = vec![
            FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full},
            FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left},
            FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right},
        ];
        let expected: f32 = 3.14;

        let mut tests_run = 0;
        for desc in descs {
            let actual: f32 = " 3.14 ".parse_with(&desc).unwrap();
            assert_eq!(actual, expected);

            let actual: f32 = "3.14  ".parse_with(&desc).unwrap();
            assert_eq!(actual, expected);

            let actual: f32 = "  3.14".parse_with(&desc).unwrap();
            assert_eq!(actual, expected);

            tests_run += 1;
        }

        assert_eq!(tests_run, 3);
    }
    */

    #[test]
    fn extract_f32_full() {
        let desc = FieldDescription{ skip: 1, len: 4, alignment: Alignment::Full};
        let actual: f32 = " 3.14 ".parse_with(&desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);

        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: Result<f32, ()> = " 3.14 ".parse_with(&desc);
        assert!(actual.is_err()); // TODO: check the error type
    }

    #[test]
    fn extract_f32_left() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Left};
        let actual: f32 = " 3.14 ".parse_with(&desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);

        let desc = FieldDescription{ skip: 2, len: 4, alignment: Alignment::Left};
        let actual: f32 = " 3.14 ".parse_with(&desc).unwrap();
        let expected: f32 = 0.14;
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_f32_right() {
        let desc = FieldDescription{ skip: 0, len: 5, alignment: Alignment::Right};
        let actual: f32 = " 3.14 ".parse_with(&desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);
    }
}
