use super::super::constants::*;
use std::fmt::Display;
use zeroize::Zeroize;

#[macro_export]
macro_rules! version_bits {
    ( $version_bits: expr ) => {
        {
            use crate::stealth::StealthAccountVersions;
            StealthAccountVersions::decode_from_bits($version_bits)
        }
    };
}

#[macro_export]
macro_rules! versions {
    ( $($version: expr),* ) => {
        {
            use crate::stealth::StealthAccountVersions;
            let mut version = StealthAccountVersions::default();
            $(
                version.enable_version($version);
            )*
            version
        }
    };
}

fn is_valid_version(version: u8) -> bool {
    version <= HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION
}

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

    pub fn supports_version(&self, version: u8) -> bool {
        if !is_valid_version(version) {
            return false
        }
        self.supported_versions[version as usize]
    }

    pub fn highest_supported_version(&self) -> Option<u8> {
        for version in (0..=HIGHEST_POSSIBLE_STEALTH_PROTOCOL_VERSION).rev() {
            if self.supports_version(version) {
                return Some(version);
            }
        }
        None
    }

    pub fn encode_to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        for i in 0..8 {
            if self.supported_versions[i] {
                bits |= 1 << i;
            }
        }
        bits
    }

    pub fn decode_from_bits(bits: u8) -> StealthAccountVersions {
        let mut versions = [false; 8];
        for i in 0..8 {
            if bits & (1 << i) != 0 {
                versions[i] = true;
            };
        }
        StealthAccountVersions::from(versions)
    }
}
impl From<[bool; 8]> for StealthAccountVersions {
    fn from(value: [bool; 8]) -> Self {
        StealthAccountVersions { supported_versions: value }
    }
}
impl From<StealthAccountVersions> for [bool; 8] {
    fn from(value: StealthAccountVersions) -> Self {
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