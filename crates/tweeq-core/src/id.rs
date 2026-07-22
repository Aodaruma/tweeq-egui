use std::fmt;

/// Stable application-owned identity for a parameter.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamId(u64);

impl ParamId {
    /// Creates an ID from an application-owned integer.
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Creates a deterministic ID from a static string using FNV-1a.
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        let bytes = value.as_bytes();
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        let mut index = 0;
        while index < bytes.len() {
            hash ^= bytes[index] as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            index += 1;
        }
        Self(hash)
    }

    /// Returns the numeric representation.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for ParamId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ParamId({:#018x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ParamId;

    #[test]
    fn static_ids_are_deterministic_and_distinct() {
        assert_eq!(
            ParamId::from_static("opacity"),
            ParamId::from_static("opacity")
        );
        assert_ne!(
            ParamId::from_static("opacity"),
            ParamId::from_static("rotation")
        );
    }
}
