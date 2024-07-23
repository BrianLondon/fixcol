
/// Represents the alignment of a field in a fixed length representation
#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    /// Field is aligned left
    Left,
    /// Field is aligned right
    Right,
    /// Field takes the full width and whitespace will not be stripped
    Full, // TODO: handle incorrect length writes (with strict mode)
}


/// Represents how a field should be encoded in fixed len representation
#[derive(Clone, Copy, Debug)]
pub struct FieldDescription {
    /// How many characters to skip between the prior field and this one
    /// 
    /// Note, currently limited to 256 for writes
    pub skip: usize,
    /// The number of characters available to hold this field
    pub len: usize,
    /// How data in this field is aligned
    pub alignment: Alignment,
}
