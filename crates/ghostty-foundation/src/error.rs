//! Shared error and result types for Rust port crates.

/// Common non-I/O failures across config, termio, and FFI boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoundationError {
    InvalidValue,
    InvalidField,
    ValueRequired,
    OutOfMemory,
    Unsupported,
}

pub type FoundationResult<T> = core::result::Result<T, FoundationError>;

impl FoundationError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidValue => "invalid value",
            Self::InvalidField => "invalid field",
            Self::ValueRequired => "value required",
            Self::OutOfMemory => "out of memory",
            Self::Unsupported => "unsupported",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FoundationError;

    #[test]
    fn error_messages_are_stable() {
        assert_eq!(FoundationError::InvalidValue.as_str(), "invalid value");
        assert_eq!(FoundationError::Unsupported.as_str(), "unsupported");
    }
}
