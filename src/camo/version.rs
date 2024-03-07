use crate::{auto_from_impl, constants::*};
use std::ops::RangeInclusive;
use zeroize::Zeroize;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Decode `CamoVersions` from the compact `u8` representation.
///
/// You propably want `versions!()` instead.
#[macro_export]
macro_rules! version_bits {
    ( $version_bits: expr ) => {{
        use $crate::camo::CamoVersions;
        CamoVersions::decode_from_bits($version_bits)
    }};
}

/// Create `CamoVersions` with all of the given versions enabled.
/// Versions which are not supported by this software will be ignored.
///
/// Note that currently, only version `1` is supported.
#[macro_export]
macro_rules! versions {
    ( $($version: expr),* ) => {
        {
            use $crate::camo::CamoVersions;
            let mut version = CamoVersions::empty();
            $(
                version.enable_version($version);
            )*
            version
        }
    };
}

fn all_possible_versions() -> RangeInclusive<u8> {
    LOWEST_POSSIBLE_CAMO_PROTOCOL_VERSION..=HIGHEST_POSSIBLE_CAMO_PROTOCOL_VERSION
}

fn all_supported_versions() -> RangeInclusive<u8> {
    LOWEST_POSSIBLE_CAMO_PROTOCOL_VERSION..=HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION
}

fn is_possible_version(version: u8) -> bool {
    all_possible_versions().contains(&version)
}

fn is_supported_version(version: u8) -> bool {
    all_supported_versions().contains(&version)
}

/// Signals the version(s) which a `camo_` account supports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroize)]
pub struct CamoVersions {
    supported_versions: [bool; 8],
}
impl CamoVersions {
    /// Create `CamoVersions` with no versions enabled
    pub fn empty() -> CamoVersions {
        CamoVersions::from([false; 8])
    }

    /// Create `CamoVersions` with all of the given versions enabled.
    /// Versions which are not supported by this software will still be set.
    pub fn new_signaling(versions: &[u8]) -> CamoVersions {
        let mut version = CamoVersions::empty();
        for i in versions {
            version.force_enable_version(*i);
        }
        version
    }

    /// Create `CamoVersions` with all of the given versions enabled.
    /// Versions which are not supported by this software will be ignored.
    ///
    /// Note that currently, only version `1` is supported.
    pub fn new(versions: &[u8]) -> CamoVersions {
        let mut version = CamoVersions::empty();
        for i in versions {
            version.enable_version(*i);
        }
        version
    }

    /// Enable the given version, regardless of whether or not that version is supported by this software
    pub fn force_enable_version(&mut self, version: u8) {
        if !is_possible_version(version) {
            return;
        }
        self.supported_versions[version as usize - 1] = true;
    }

    /// Enable the given version, so long as that version is supported by this software.
    /// Returns whether or not the version was enabled.
    pub fn enable_version(&mut self, version: u8) -> bool {
        if !is_supported_version(version) {
            return false;
        }
        self.force_enable_version(version);
        true
    }

    /// Disable the given version
    pub fn disable_version(&mut self, version: u8) {
        if !is_possible_version(version) {
            return;
        }
        self.supported_versions[version as usize - 1] = false;
    }

    /// Returns whether or not the given version is supported by the `camo_` account **but** not necessarily supported by this software
    pub fn signals_version(&self, version: u8) -> bool {
        if !is_possible_version(version) {
            return false;
        }
        self.supported_versions[version as usize - 1]
    }

    /// Returns whether or not the given version is supported by the `camo_` account **and** supported by this software
    pub fn supports_version(&self, version: u8) -> bool {
        if !is_supported_version(version) {
            return false;
        }
        self.signals_version(version)
    }

    /// Returns the highest version that is supported by the `camo_` account **but** not necessarily supported by this software
    pub fn highest_signaled_version(&self) -> Option<u8> {
        all_possible_versions()
            .rev()
            .find(|&version| self.signals_version(version))
    }

    /// Returns the highest version that is supported by the `camo_` account **and** supported by this software
    pub fn highest_supported_version(&self) -> Option<u8> {
        all_possible_versions()
            .rev()
            .find(|&version| self.supports_version(version))
    }

    /// Returns all versions that are supported by the `camo_` account **but** not necessarily supported by this software
    pub fn all_signaled_versions(&self) -> Vec<u8> {
        all_possible_versions()
            .filter(|version| self.signals_version(*version))
            .collect()
    }

    /// Returns all versions that are supported by the `camo_` account **and** supported by this software
    pub fn all_supported_versions(&self) -> Vec<u8> {
        all_possible_versions()
            .filter(|version| self.supports_version(*version))
            .collect()
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
    pub fn decode_from_bits(bits: u8) -> CamoVersions {
        let mut versions = [false; 8];
        for (i, version) in versions.iter_mut().enumerate() {
            if bits & (1 << i) != 0 {
                *version = true;
            };
        }
        CamoVersions::from(versions)
    }
}
#[cfg(feature = "serde")]
impl Serialize for CamoVersions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.encode_to_bits().serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CamoVersions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(CamoVersions::decode_from_bits(u8::deserialize(
            deserializer,
        )?))
    }
}
auto_from_impl!(From: [bool; 8] => CamoVersions);
auto_from_impl!(From: CamoVersions => [bool; 8]);
impl From<&[bool; 8]> for CamoVersions {
    fn from(value: &[bool; 8]) -> Self {
        CamoVersions {
            supported_versions: *value,
        }
    }
}
impl From<&CamoVersions> for [bool; 8] {
    fn from(value: &CamoVersions) -> Self {
        value.supported_versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VERSIONS_1: CamoVersions = CamoVersions {
        supported_versions: [true, false, true, false, true, true, false, false],
    };
    const TEST_VERSIONS_2: CamoVersions = CamoVersions {
        supported_versions: [false, true, false, false, true, true, false, true],
    };
    const TEST_VERSIONS_3: CamoVersions = CamoVersions {
        supported_versions: [true, true, true, true, true, true, true, true],
    };

    #[test]
    fn valid_versions() {
        assert!(!is_possible_version(0));
        assert!(!is_supported_version(0));

        for i in 1..=HIGHEST_POSSIBLE_CAMO_PROTOCOL_VERSION {
            assert!(is_possible_version(i));
        }
        for i in 1..=HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION {
            assert!(is_supported_version(i));
        }

        assert!(!is_possible_version(9));
        assert!(!is_supported_version(9));
    }

    #[test]
    fn highest_signaled_version() {
        assert!(TEST_VERSIONS_1.highest_signaled_version() == Some(6));
        assert!(TEST_VERSIONS_2.highest_signaled_version() == Some(8));
        assert!(TEST_VERSIONS_3.highest_signaled_version() == Some(8));
    }

    #[test]
    fn highest_supported_version() {
        assert!(TEST_VERSIONS_1.highest_supported_version() == Some(1));
        assert!(TEST_VERSIONS_2.highest_supported_version().is_none());
        assert!(TEST_VERSIONS_3.highest_supported_version() == Some(1));
    }

    #[test]
    fn all_signaled_versions() {
        assert!(TEST_VERSIONS_1.all_signaled_versions() == vec!(1, 3, 5, 6));
        assert!(TEST_VERSIONS_2.all_signaled_versions() == vec!(2, 5, 6, 8));
        assert!(TEST_VERSIONS_3.all_signaled_versions() == vec!(1, 2, 3, 4, 5, 6, 7, 8));
    }

    #[test]
    fn all_supported_versions() {
        assert!(TEST_VERSIONS_1.all_supported_versions() == vec!(1));
        assert!(TEST_VERSIONS_2.all_supported_versions().is_empty());
        assert!(TEST_VERSIONS_3.all_supported_versions() == vec!(1));
    }

    #[test]
    fn to_bits() {
        assert!(TEST_VERSIONS_1.encode_to_bits() == 0b_0011_0101);
        assert!(TEST_VERSIONS_2.encode_to_bits() == 0b_1011_0010);
        assert!(TEST_VERSIONS_3.encode_to_bits() == 0b_1111_1111);
    }

    #[test]
    fn from_bits() {
        let versions_1 = versions!(HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION);
        let versions_2 = CamoVersions::decode_from_bits(versions_1.encode_to_bits());
        assert!(versions_1 == versions_2);

        assert!(
            CamoVersions::decode_from_bits(TEST_VERSIONS_1.encode_to_bits()) == TEST_VERSIONS_1
        );
        assert!(
            CamoVersions::decode_from_bits(TEST_VERSIONS_2.encode_to_bits()) == TEST_VERSIONS_2
        );
        assert!(
            CamoVersions::decode_from_bits(TEST_VERSIONS_3.encode_to_bits()) == TEST_VERSIONS_3
        );
    }
}
