
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
    pub skip: usize,
    pub len: usize,
    pub alignment: Alignment,
}
