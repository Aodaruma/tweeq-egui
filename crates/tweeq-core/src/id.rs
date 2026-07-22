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

    /// Derives a stable child ID for a component or sub-control.
    #[must_use]
    pub const fn child(self, discriminator: u64) -> Self {
        let mixed = self.0 ^ discriminator.wrapping_add(0x9e37_79b9_7f4a_7c15);
        Self(mixed.wrapping_mul(0xbf58_476d_1ce4_e5b9))
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
