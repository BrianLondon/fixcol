use std::str::FromStr;

/// Represents the alignment of a field in a fixed length representation
pub enum Alignment {
    /// Field is aligned left
    Left,
    /// Field is aligned right
    Right,
    /// Field takes the full width and whitespace will not be stripped
    Full, // TODO: handle incorrect length writes (with strict mode)
}


/// Represents how a field should be encoded in fixed len representation
pub struct EncodingScheme {
    pub skip: usize,
    pub len: usize,
    pub alignment: Alignment,
}


/// A trait the represents field types that can be decoded from fixed len strings
pub trait FixedDeserializable<T : Sized> {
    fn parse_with(&self, scheme: EncodingScheme) -> Result<T, ()>;
}


fn extract_trimmed(src: &str, scheme: EncodingScheme) -> &str {
    let slice = &src[scheme.skip..scheme.skip+scheme.len];
        
    match scheme.alignment {
        Alignment::Left => slice.trim_end(),
        Alignment::Right => slice.trim_start(),
        Alignment::Full => slice,
    }
}


impl<T: FromStr> FixedDeserializable<T> for &str {
    fn parse_with(&self, scheme: EncodingScheme) -> Result<T, ()> {
        let trimmed = extract_trimmed(self, scheme);
        trimmed.parse::<T>().map_err(|_| ())
    }
}
