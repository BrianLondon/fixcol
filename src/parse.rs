use crate::format::{Alignment, FieldDescription};

use std::str::FromStr;


/// A trait the represents field types that can be decoded from fixed len strings
pub trait FixedDeserializable<T : Sized> {
    fn parse_with(&self, desc: FieldDescription) -> Result<T, ()>;
}


fn extract_trimmed(src: &str, desc: FieldDescription) -> &str {
    let slice = &src[desc.skip..desc.skip+desc.len];
        
    match desc.alignment {
        Alignment::Left => slice.trim_end(),
        Alignment::Right => slice.trim_start(),
        Alignment::Full => slice,
    }
}


impl<T: FromStr> FixedDeserializable<T> for &str {
    fn parse_with(&self, desc: FieldDescription) -> Result<T, ()> {
        let trimmed = extract_trimmed(self, desc);
        trimmed.parse::<T>().map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_string_left() {
        let desc = FieldDescription{ skip: 0, len: 3, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_pad() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_truncate() {
        let desc = FieldDescription{ skip: 0, len: 2, alignment: Alignment::Left};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = "a bc  ".parse_with(desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_left_leading_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Left};
        let actual: String = " abc  ".parse_with(desc).unwrap();
        let expected = " abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_exact() {
        let desc = FieldDescription{ skip: 0, len: 3, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "abc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_skip_into() {
        let desc = FieldDescription{ skip: 4, len: 2, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_truncate() {
        let desc = FieldDescription{ skip: 1, len: 4, alignment: Alignment::Right};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "ab".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = "  a bc".parse_with(desc).unwrap();
        let expected = "a bc".to_string();
        assert_eq!(actual, expected)
    }

    #[test]
    fn extract_string_right_trailing_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Right};
        let actual: String = " abc  ".parse_with(desc).unwrap();
        let expected = "abc  ".to_string();
        assert_eq!(actual, expected)
    }


    #[test]
    fn extract_string_full() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "abcdef".parse_with(desc).unwrap();
        let expected = "abcdef".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_slice() {
        let desc = FieldDescription{ skip: 1, len: 3, alignment: Alignment::Full};
        let actual: String = "abcdef".parse_with(desc).unwrap();
        let expected = "bcd".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_left() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "abc   ".to_string();
        assert_eq!(actual, expected);
    }


    #[test]
    fn extract_string_full_right() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = "   abc".parse_with(desc).unwrap();
        let expected = "   abc".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_skip() {
        let desc = FieldDescription{ skip: 1, len: 5, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "bc   ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_truncate() {
        let desc = FieldDescription{ skip: 0, len: 4, alignment: Alignment::Full};
        let actual: String = "abc   ".parse_with(desc).unwrap();
        let expected = "abc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_ws() {
        let desc = FieldDescription{ skip: 0, len: 6, alignment: Alignment::Full};
        let actual: String = " a bc ".parse_with(desc).unwrap();
        let expected = " a bc ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_trimmed_ws() {
        let desc = FieldDescription{ skip: 1, len: 3, alignment: Alignment::Full};
        let actual: String = " ab c ".parse_with(desc).unwrap();
        let expected = "ab ".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_string_full_tight_wc() {
        let desc = FieldDescription{ skip: 1, len: 4, alignment: Alignment::Full};
        let actual: String = " ab c ".parse_with(desc).unwrap();
        let expected = "ab c".to_string();
        assert_eq!(actual, expected);
    }
}