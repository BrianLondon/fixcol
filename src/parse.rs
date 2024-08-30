use crate::error::{DataError, Error, InnerError};
use crate::format::{Alignment, FieldDescription};
use crate::ReadFixed;

/// A trait the represents field types that can be decoded from fixed length strings
///
/// Implementations are provided for `&str` that are used internally in the macro
/// generated code to derive [`ReadFixed`]. It is unlikely end users will need to
/// implement this trait for other types.
///
/// It may be useful to implement this for other `T` on &str if you would like
/// to directly deserialize other primitives.
///
/// Additionally, it is possible to use custom `FixedDeserializer` implementations
/// with the [new type](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
/// pattern to define custom derserialization logic.
///
/// Custom error messages can be created with [`DataError::custom`].
///
/// [ReadFixed]: crate::ReadFixed
///
/// # Examples
///
/// ### Custom deserialization
///
/// ```
/// # use fixcol::ReadFixed;
/// # use fixcol::FixedDeserializer;
/// # use fixcol::FieldDescription;
/// # use fixcol::error::DataError;
/// #[derive(PartialEq, Eq, Debug)]
/// enum EyeColor {
///     Blue,
///     Brown,
///     Green,
/// }
///
/// impl FixedDeserializer for EyeColor {
///     fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<EyeColor, DataError> {
///         match s {
///             "Bl" => Ok(EyeColor::Blue),
///             "Br" => Ok(EyeColor::Brown),
///             "Gr" => Ok(EyeColor::Green),
///             _ => Err(DataError::custom(s, "Unrecognized eye color")),
///         }
///     }
/// }
///
/// #[derive(ReadFixed)]
/// struct Person {
///     #[fixcol(width = 10)]
///     pub name: String,
///     #[fixcol(width=3, align=right)]
///     pub age: u8,
///     #[fixcol(width = 2)]
///     pub eye_color: EyeColor,
/// }
///
/// let person = Person::read_fixed_str("Harold     42Gr").unwrap();
/// assert_eq!(person.eye_color, EyeColor::Green);
/// ```
///
/// ### Multiple deserialization approached
///
/// Here we use a few different approaches to deserializing a fixed column
/// data file. Documentation of the file structure follows.
///
/// ```text
///     Name      Birthday
/// /----------\ /--------\
/// XXXXXXXXXXXX YYYY MM DD
///       \        \   \  \--- Day   (numeric)
///        \        \   \----- Month (numeric)
///         \        \-------- Year  (numeric)
///          \
///           \--------------- Name  (alphabetic)
///
/// Example rows
/// George       1989  3 12     
/// Claire       2001 11 26
/// ```
///
/// Naive implementation
///
/// ```
/// # use fixcol::ReadFixed;
/// # use fixcol::FixedDeserializer;
/// # use fixcol::FieldDescription;
/// # use std::fs::File;
/// #[derive(ReadFixed)]
/// # #[derive(Eq, PartialEq, Debug)]
/// struct Person {
///     #[fixcol(width=12)]
///     name: String,
///     #[fixcol(width=4, skip=1, align="right")]
///     birth_year: u16,
///     #[fixcol(width=2, skip=1, align="right")]
///     birth_month: u8,
///     #[fixcol(width=2, skip=1, align="right")]
///     birth_date: u8,
/// }
///
/// // Note we are being sloppy with error handling to keep the example simple
/// # fn f() {
/// let mut file = File::open("my_file.txt").unwrap();
/// # }
/// # let mut file = "George       1989  3 12\nClaire       2001 11 26".as_bytes();
/// let people: Vec<Person> = Person::read_fixed_all(file)
///     .map(|res| res.unwrap())
///     .collect();
/// # assert_eq!(people, vec![
/// #     Person{name: "George".to_string(), birth_year: 1989, birth_month: 3, birth_date: 12},
/// #     Person{name: "Claire".to_string(), birth_year: 2001, birth_month: 11, birth_date: 26},
/// # ]);
/// ```
///
/// Same data file, but this time using a custom `FixedDeserializer` to decode the date.
/// We use a `Birthday` new type around a [`chrono::NaiveDate`].
///
/// [`chrono::NaiveDate`]: https://docs.rs/chrono/latest/chrono/struct.NaiveDate.html
///
/// ```
/// # use fixcol::ReadFixed;
/// # use fixcol::FixedDeserializer;
/// # use fixcol::FieldDescription;
/// # use fixcol::error::DataError;
/// # use std::fs::File;
/// #[derive(ReadFixed)]
/// # #[derive(Eq, PartialEq, Debug)]
/// struct Person {
///     #[fixcol(width = 12)]
///     name: String,
///     #[fixcol(width = 10, skip = 1)]
///     birthday: Birthday,
/// }
///
/// use chrono::NaiveDate;
/// # #[derive(Eq, PartialEq, Debug)]
/// struct Birthday(NaiveDate);
///
/// impl FixedDeserializer for Birthday {
///     fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<Birthday, DataError> {
///         let text = &s[desc.skip..desc.skip + desc.len];
///         let mut parts = text.split(' ').filter(|x| *x != "");
///
///         let year = parts
///             .next()
///             .ok_or(DataError::custom(&text, "Could not find year"))?
///             .parse()
///             .map_err(|e| DataError::custom(&text, "Could not decode year"))?;
///
///         let month = parts
///             .next()
///             .ok_or(DataError::custom(&text, "Could not find month"))?
///             .parse()
///             .map_err(|e| DataError::custom(&text, "Could not decode month"))?;
///
///         let day = parts
///             .next()
///             .ok_or(DataError::custom(&text, "Could not find day"))?
///             .parse()
///             .map_err(|e| DataError::custom(&text, "Could not decode day"))?;
///
///         let dt = NaiveDate::from_ymd_opt(year, month, day)
///             .ok_or(DataError::custom(&text, "Did not recognize a date"))?;
///
///         Ok(Birthday(dt))
///     }
/// }
///
/// # fn f() {
/// let mut file = File::open("my_file.txt").unwrap();
/// # }
/// # let mut file = "George       1989  3 12\nClaire       2001 11 26".as_bytes();
/// let people: Vec<Person> = Person::read_fixed_all(file)
///     .map(|res| res.unwrap())
///     .collect();
/// # assert_eq!(people, vec![
/// #     Person {
/// #         name: "George".to_string(),
/// #         birthday: Birthday(NaiveDate::from_ymd_opt(1989, 3, 12).unwrap())
/// #     },
/// #     Person {
/// #         name: "Claire".to_string(),
/// #         birthday: Birthday(NaiveDate::from_ymd_opt(2001, 11, 26).unwrap())
/// #     },
/// # ]);
/// ```
pub trait FixedDeserializer {
    /// Read an object of type `T` from the current object.
    ///
    /// Uses the provided [`FieldDescription`] to determine how to parse a data field
    /// from a fixed column representation.
    fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<Self, DataError>
    where
        Self: Sized;
}

fn extract_trimmed<'a, 'b>(src: &'a str, desc: &'b FieldDescription) -> Result<&'a str, DataError> {
    if desc.strict && !&src[..desc.skip].trim().is_empty() {
        return Err(DataError::whitespace_error(String::from(src)));
    }

    let end = std::cmp::min(desc.skip + desc.len, src.len());

    let slice = &src[desc.skip..end];

    let res = match (desc.strict, desc.alignment) {
        (true, Alignment::Left) => slice.trim_end(),
        (true, Alignment::Right) => slice.trim_start(),
        (true, Alignment::Full) => slice,
        _ => slice.trim_start().trim_end(),
    };

    Ok(res)
}

macro_rules! fixed_deserializer_float_impl {
    ($t:ty) => {
        impl FixedDeserializer for $t {
            fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<$t, DataError> {
                let trimmed = extract_trimmed(s, desc)?;
                trimmed.parse::<$t>().map_err(|e| {
                    DataError::new_err(trimmed.to_string(), InnerError::ParseFloatError(e))
                })
            }
        }
    };
}

fixed_deserializer_float_impl!(f32);
fixed_deserializer_float_impl!(f64);

macro_rules! fixed_deserializer_int_impl {
    ($t:ty) => {
        impl FixedDeserializer for $t {
            fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<$t, DataError> {
                let trimmed = extract_trimmed(s, desc)?;

                if desc.strict && desc.alignment == Alignment::Full && trimmed.len() != s.len() {
                    let trimmed_len = trimmed.len();
                    Err(DataError::new_data_width_error(
                        String::from(trimmed),
                        trimmed_len,
                        s.len(),
                    ))
                } else {
                    trimmed.parse::<$t>().map_err(|e| {
                        DataError::new_err(trimmed.to_string(), InnerError::ParseIntError(e))
                    })
                }
            }
        }
    };
}

fixed_deserializer_int_impl!(u8);
fixed_deserializer_int_impl!(u16);
fixed_deserializer_int_impl!(u32);
fixed_deserializer_int_impl!(u64);
fixed_deserializer_int_impl!(u128);

fixed_deserializer_int_impl!(i8);
fixed_deserializer_int_impl!(i16);
fixed_deserializer_int_impl!(i32);
fixed_deserializer_int_impl!(i64);
fixed_deserializer_int_impl!(i128);

fixed_deserializer_int_impl!(usize);
fixed_deserializer_int_impl!(isize);

impl FixedDeserializer for String {
    fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<String, DataError> {
        let slice = &s[desc.skip..desc.skip + desc.len];

        let trimmed = match desc.alignment {
            Alignment::Left => slice.trim_end(),
            Alignment::Right => slice.trim_start(),
            Alignment::Full => slice,
        };

        Ok(trimmed.to_string())
    }
}

impl<T: ReadFixed> FixedDeserializer for T {
    fn parse_fixed(s: &str, desc: &FieldDescription) -> Result<Self, DataError> {
        let slice = &s[desc.skip..desc.skip + desc.len];

        let obj = T::read_fixed_str(slice).map_err(|e| match e {
            Error::DataError(e) => e,
            Error::IoError(e) => {
                panic!("I/O error while reading internal memory: {:?}", e);
            }
        })?;

        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;

    #[test]
    fn extract_string_left() {
        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_pad() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_skip() {
        let desc = FieldDescription {
            skip: 1,
            len: 5,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_truncate() {
        let desc = FieldDescription {
            skip: 0,
            len: 2,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_ws() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed("a bc  ", &desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_leading_ws() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: String = String::parse_fixed(" abc  ", &desc).unwrap();
        let expected = " abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_exact() {
        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip() {
        let desc = FieldDescription {
            skip: 1,
            len: 5,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip_into() {
        let desc = FieldDescription {
            skip: 4,
            len: 2,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_truncate() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_ws() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed("  a bc", &desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_trailing_ws() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: String = String::parse_fixed(" abc  ", &desc).unwrap();
        let expected = "abc  ".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_full() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("abcdef", &desc).unwrap();
        let expected = "abcdef".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_slice() {
        let desc = FieldDescription {
            skip: 1,
            len: 3,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("abcdef", &desc).unwrap();
        let expected = "bcd".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_left() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "abc   ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_right() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("   abc", &desc).unwrap();
        let expected = "   abc".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_skip() {
        let desc = FieldDescription {
            skip: 1,
            len: 5,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "bc   ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_truncate() {
        let desc = FieldDescription {
            skip: 0,
            len: 4,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed("abc   ", &desc).unwrap();
        let expected = "abc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_ws() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed(" a bc ", &desc).unwrap();
        let expected = " a bc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_trimmed_ws() {
        let desc = FieldDescription {
            skip: 1,
            len: 3,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed(" ab c ", &desc).unwrap();
        let expected = "ab ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_tight_wc() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual: String = String::parse_fixed(" ab c ", &desc).unwrap();
        let expected = "ab c".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_f32_padding() {
        let descs = vec![
            FieldDescription {
                skip: 0,
                len: 6,
                alignment: Alignment::Full,
                strict: false,
            },
            FieldDescription {
                skip: 0,
                len: 6,
                alignment: Alignment::Left,
                strict: false,
            },
            FieldDescription {
                skip: 0,
                len: 6,
                alignment: Alignment::Right,
                strict: false,
            },
        ];
        let expected: f32 = 3.14;

        let mut tests_run = 0;
        for desc in descs {
            println!("a {:?}", desc);
            let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
            assert_eq!(actual, expected);
            println!("b {:?}", desc);

            let actual: f32 = f32::parse_fixed("3.14  ", &desc).unwrap();
            assert_eq!(actual, expected);
            println!("c {:?}", desc);

            let actual: f32 = f32::parse_fixed("  3.14", &desc).unwrap();
            assert_eq!(actual, expected);
            println!("d {:?}", desc);

            tests_run += 1;
        }

        assert_eq!(tests_run, 3);
    }

    #[test]
    fn extract_f32_full() {
        let desc = FieldDescription {
            skip: 1,
            len: 4,
            alignment: Alignment::Full,
            strict: true,
        };
        let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);

        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Full,
            strict: true,
        };
        let actual: Result<f32, DataError> = f32::parse_fixed(" 3.14 ", &desc);

        match actual {
            Ok(_) => panic!("Expected parse_fixed call to fail"),
            Err(e) => match e.inner_error() {
                InnerError::ParseFloatError(_) => {}
                _ => panic!("Expected ParseFloatError as inner error"),
            },
        }
    }

    #[test]
    fn extract_f32_left() {
        let desc = FieldDescription {
            skip: 1,
            len: 5,
            alignment: Alignment::Left,
            strict: true,
        };
        let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);

        let desc = FieldDescription {
            skip: 2,
            len: 4,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
        let expected: f32 = 0.14;
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_f32_right() {
        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Right,
            strict: true,
        };
        let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_f32_right_strict() {
        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: f32 = f32::parse_fixed(" 3.14 ", &desc).unwrap();
        let expected: f32 = 3.14;
        assert_eq!(actual, expected);

        let desc = FieldDescription {
            skip: 0,
            len: 6,
            alignment: Alignment::Right,
            strict: true,
        };
        match f32::parse_fixed(" 3.14 ", &desc) {
            Ok(_) => panic!("Expected parse_fixed call to fail"),
            Err(e) => match e.inner_error() {
                InnerError::ParseFloatError(_) => {}
                _ => panic!("Expected ParseFloatError as inner error"),
            },
        }
    }

    #[test]
    fn extract_f32_bad() {
        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual: Result<f32, DataError> = f32::parse_fixed(" 3a14 ", &desc);
        let expected = "Error decoding data from \"3a14\": invalid float literal\n";
        assert_eq!(actual.unwrap_err().to_string(), expected);
    }

    #[test]
    fn strict_numeric_zero_padding() {
        // validate "strict" behavior
        // require no whitespace in numeric `Full` columns
        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual = u8::parse_fixed("042", &desc).unwrap();
        assert_eq!(actual, 42);

        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Full,
            strict: true,
        };
        let actual = u8::parse_fixed("042", &desc).unwrap();
        assert_eq!(actual, 42);

        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Full,
            strict: false,
        };
        let actual = u8::parse_fixed(" 42", &desc).unwrap();
        assert_eq!(actual, 42);

        let desc = FieldDescription {
            skip: 0,
            len: 3,
            alignment: Alignment::Full,
            strict: true,
        };
        let actual = u8::parse_fixed(" 42", &desc);
        assert!(actual.is_err());
        assert_eq!(
            actual.unwrap_err().to_string(),
            "Error decoding data from \" 42\": invalid digit found in string\n"
        );
    }

    #[test]
    fn strict_left_align() {
        // testing "strict" behavior
        // left aligned fields cannot start with white space
        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Left,
            strict: false,
        };
        let actual = u8::parse_fixed(" 42  ", &desc).unwrap();
        assert_eq!(actual, 42);

        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Left,
            strict: true,
        };
        let actual = u8::parse_fixed(" 42  ", &desc);
        assert!(actual.is_err());
        assert_eq!(
            actual.unwrap_err().to_string(),
            "Error decoding data from \" 42\": invalid digit found in string\n"
        );

        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Left,
            strict: true,
        };
        let actual = u8::parse_fixed("42   ", &desc).unwrap();
        assert_eq!(actual, 42);
    }

    #[test]
    fn strict_right_align() {
        // testing "strict" behavior:
        // right aligned fields cannot end with white space
        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Right,
            strict: false,
        };
        let actual = u8::parse_fixed("  42 ", &desc).unwrap();
        assert_eq!(actual, 42);

        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Right,
            strict: true,
        };
        let actual = u8::parse_fixed("  42 ", &desc);
        assert!(actual.is_err());
        assert_eq!(
            actual.unwrap_err().to_string(),
            "Error decoding data from \"42 \": invalid digit found in string\n"
        );

        let desc = FieldDescription {
            skip: 0,
            len: 5,
            alignment: Alignment::Right,
            strict: true,
        };
        let actual = u8::parse_fixed("   42", &desc).unwrap();
        assert_eq!(actual, 42);
    }

    #[test]
    fn impl_parse() {
        #[derive(PartialEq, Eq, Debug)]
        enum Thing {
            Thing1,
            Thing2,
        }

        impl ReadFixed for Thing {
            fn read_fixed<R>(buf: &mut R) -> Result<Self, Error>
            where
                Self: Sized,
                R: std::io::Read,
            {
                let mut v: [u8; 2] = [0; 2];
                let res = buf.read_exact(&mut v);
                assert!(res.is_ok());
                let s = from_utf8(v.as_slice()).unwrap();

                match s {
                    "T1" => Ok(Self::Thing1),
                    "T2" => Ok(Self::Thing2),
                    x => Err(DataError::custom(x, "failed").into()),
                }
            }
        }

        let thing = Thing::parse_fixed(
            " T1 ",
            &FieldDescription {
                skip: 1,
                len: 2,
                alignment: Alignment::Left,
                strict: true,
            },
        );

        assert_eq!(thing.unwrap(), Thing::Thing1);
    }
}
