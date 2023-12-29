use crate::{auto_from_impl, constants::*};
use std::fmt::Display;
use zeroize::Zeroize;

#[macro_export]
macro_rules! version_bits {
    ( $version_bits: expr ) => {
        {
            use $crate::stealth::StealthAccountVersions;
            StealthAccountVersions::decode_from_bits($version_bits)
        }
    };
}

#[macro_export]
macro_rules! versions {
    ( $($version: expr),* ) => {
        {
            use $crate::stealth::StealthAccountVersions;
            let mut version = StealthAccountVersions::default();
            $(
                version.enable_version($version);
            )*
            version
        }
    };
}

#[allow(clippy::absurd_extreme_comparisons)]
fn is_valid_version(version: u8) -> bool {
    version <= HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION
}

/// Signals the version(s) which a `stealth_` account supports
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Zeroize)]
pub struct StealthAccountVersions {
    supported_versions: [bool; 8]
}
impl StealthAccountVersions {
    pub fn new(versions: Vec<u8>) -> StealthAccountVersions {
        let mut version = StealthAccountVersions::default();
        for i in versions {
            version.enable_version(i)
        }
        version
    }

    pub fn enable_version(&mut self, version: u8) {
        if !is_valid_version(version) {
            return;
        }
        self.supported_versions[version as usize] = true;
    }

    pub fn disable_version(&mut self, version: u8) {
        if !is_valid_version(version) {
            return;
        }
        self.supported_versions[version as usize] = false;
    }

    /// Returns whether the `stealth_` account supports the version.
    ///
    /// Will always return `false` if this software does not support the given version.
    pub fn supports_version(&self, version: u8) -> bool {
        if !is_valid_version(version) {
            return false
        }
        self.supported_versions[version as usize]
    }

    /// Returns the highest version that is supported by both the `stealth_` account and this software
    pub fn highest_supported_version(&self) -> Option<u8> {
        (0..=HIGHEST_POSSIBLE_STEALTH_PROTOCOL_VERSION)
            .rev()
            .find(|&version| self.supports_version(version))
    }

    /// Encode the version support to a `u8`
    pub fn encode_to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        for (i, supports_version) in self.supported_versions.into_iter().enumerate() {
            if supports_version {
                bits |= 1 << i;
            }
        }
        bits
    }

    /// Decode the version support from a `u8`
    pub fn decode_from_bits(bits: u8) -> StealthAccountVersions {
        let mut versions = [false; 8];
        for (i, version) in versions.iter_mut().enumerate() {
            if bits & (1 << i) != 0 {
                *version = true;
            };
        }
        StealthAccountVersions::from(versions)
    }
}
auto_from_impl!(From, [bool; 8], StealthAccountVersions);
auto_from_impl!(From, StealthAccountVersions, [bool; 8]);
impl From<&[bool; 8]> for StealthAccountVersions {
    fn from(value: &[bool; 8]) -> Self {
        StealthAccountVersions { supported_versions: *value }
    }
}
impl From<&StealthAccountVersions> for [bool; 8] {
    fn from(value: &StealthAccountVersions) -> Self {
        value.supported_versions
    }
}
impl Display for StealthAccountVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.supported_versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VERSIONS_1: StealthAccountVersions = StealthAccountVersions {
        supported_versions: [true, false, true, false, true, true, false, false]
    };
    const TEST_VERSIONS_2: StealthAccountVersions = StealthAccountVersions {
        supported_versions: [false, true, false, false, true, true, false, true]
    };
    const TEST_VERSIONS_3: StealthAccountVersions = StealthAccountVersions {
        supported_versions: [true, true, true, true, true, true, true, true]
    };

    #[test]
    fn highest_supported_version() {
        assert!(TEST_VERSIONS_1.highest_supported_version() == Some(0));
        assert!(TEST_VERSIONS_2.highest_supported_version() == None);
        assert!(TEST_VERSIONS_3.highest_supported_version() == Some(0));
    }

    #[test]
    fn to_bits() {
        assert!(TEST_VERSIONS_1.encode_to_bits() == 0b_0011_0101);
        assert!(TEST_VERSIONS_2.encode_to_bits() == 0b_1011_0010);
        assert!(TEST_VERSIONS_3.encode_to_bits() == 0b_1111_1111);
    }

    #[test]
    fn from_bits() {
        let versions_1 = versions!(HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION);
        let versions_2 = StealthAccountVersions::decode_from_bits(versions_1.encode_to_bits());
        assert!(versions_1 == versions_2);

        assert!(StealthAccountVersions::decode_from_bits(TEST_VERSIONS_1.encode_to_bits()) == TEST_VERSIONS_1);
        assert!(StealthAccountVersions::decode_from_bits(TEST_VERSIONS_2.encode_to_bits()) == TEST_VERSIONS_2);
        assert!(StealthAccountVersions::decode_from_bits(TEST_VERSIONS_3.encode_to_bits()) == TEST_VERSIONS_3);
    }
}