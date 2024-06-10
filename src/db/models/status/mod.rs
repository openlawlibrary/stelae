//! Utility enum representation for the status of a document or library.

/// Enum representation for the status of a document or library.
pub enum Status {
    /// 0 - Element added.
    ElementAdded,
    /// 1 - Element changed.
    ElementChanged,
    /// 2 - Element removed.
    ElementRemoved,
    /// 3 - Element effective.
    ElementEffective,
}

impl Status {
    /// Convert a string to a `Status` enum.
    /// # Errors
    /// Returns an error if the string is not a valid status value.
    pub fn from_string(status: &str) -> anyhow::Result<Self> {
        match status {
            "Element added" => Ok(Self::ElementAdded),
            "Element changed" => Ok(Self::ElementChanged),
            "Element removed" => Ok(Self::ElementRemoved),
            "Element effective" => Ok(Self::ElementEffective),
            _ => Err(anyhow::anyhow!("Invalid status value")),
        }
    }

    /// Convert a `Status` enum to an integer.
    #[must_use]
    pub const fn to_int(&self) -> i64 {
        match *self {
            Self::ElementAdded => 0,
            Self::ElementEffective => 1,
            Self::ElementChanged => 2,
            Self::ElementRemoved => 3,
        }
    }
}
