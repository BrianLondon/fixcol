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
