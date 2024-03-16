use crate::{auto_from_impl, constants::*, NanoError};
use zeroize::Zeroize;

#[cfg(feature = "serde")]
use serde::{de::Error as SerdeError, Deserialize, Serialize};

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
            use $crate::camo::{CamoVersions};
            let mut version = CamoVersions::empty();
            $(
                if let Ok(v) = $version.try_into() {
                    version.enable_version(v);
                }
            )*
            version
        }
    };
}

fn is_possible_version(version: u8) -> bool {
    match version.try_into() {
        Ok(v) => ALL_POSSIBLE_CAMO_VERSIONS.contains(&v),
        Err(_) => false,
    }
}

fn is_supported_version(version: u8) -> bool {
    match version.try_into() {
        Ok(v) => ALL_SUPPORTED_CAMO_VERSIONS.contains(&v),
        Err(_) => false,
    }
}

/// A Camo protocol version
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroize)]
pub enum CamoVersion {
    /// Camo protocol version 1 (currently the only implemented version)
    One = 1,
    /// Camo protocol version 2 (unimplemented)
    Two = 2,
    /// Camo protocol version 3 (unimplemented)
    Three = 3,
    /// Camo protocol version 4 (unimplemented)
    Four = 4,
    /// Camo protocol version 5 (unimplemented)
    Five = 5,
    /// Camo protocol version 6 (unimplemented)
    Six = 6,
    /// Camo protocol version 7 (unimplemented)
    Seven = 7,
    /// Camo protocol version 8 (unimplemented)
    Eight = 8,
}
impl CamoVersion {
    pub fn as_u8(&self) -> u8 {
        self.into()
    }
}
#[cfg(feature = "serde")]
impl Serialize for CamoVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let byte: u8 = self.into();
        byte.serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CamoVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version =
            CamoVersion::try_from(u8::deserialize(deserializer)?).map_err(SerdeError::custom)?;
        Ok(version)
    }
}
auto_from_impl!(TryFrom: u8 => CamoVersion);
auto_from_impl!(From: CamoVersion => u8);
impl TryFrom<&u8> for CamoVersion {
    type Error = NanoError;
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(CamoVersion::One),
            2 => Ok(CamoVersion::Two),
            3 => Ok(CamoVersion::Three),
            4 => Ok(CamoVersion::Four),
            5 => Ok(CamoVersion::Five),
            6 => Ok(CamoVersion::Six),
            7 => Ok(CamoVersion::Seven),
            8 => Ok(CamoVersion::Eight),
            _ => Err(NanoError::IncompatibleCamoVersions),
        }
    }
}
impl From<&CamoVersion> for u8 {
    fn from(value: &CamoVersion) -> Self {
        match value {
            CamoVersion::One => 1,
            CamoVersion::Two => 2,
            CamoVersion::Three => 3,
            CamoVersion::Four => 4,
            CamoVersion::Five => 5,
            CamoVersion::Six => 6,
            CamoVersion::Seven => 7,
            CamoVersion::Eight => 8,
        }
    }
}
impl PartialEq<u8> for CamoVersion {
    fn eq(&self, other: &u8) -> bool {
        let as_u8: u8 = self.into();
        other == &as_u8
    }
}
impl PartialEq<CamoVersion> for u8 {
    fn eq(&self, other: &CamoVersion) -> bool {
        other == self
    }
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
    pub fn new_signaling(versions: &[CamoVersion]) -> CamoVersions {
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
    pub fn new(versions: &[CamoVersion]) -> CamoVersions {
        let mut version = CamoVersions::empty();
        for i in versions {
            version.enable_version(*i);
        }
        version
    }

    /// Enable the given version, regardless of whether or not that version is supported by this software
    pub fn force_enable_version(&mut self, version: CamoVersion) {
        self.supported_versions[version.as_u8() as usize - 1] = true;
    }

    /// Enable the given version, so long as that version is supported by this software.
    /// Returns whether or not the version was enabled.
    pub fn enable_version(&mut self, version: CamoVersion) -> bool {
        if !is_supported_version(version.as_u8()) {
            return false;
        }
        self.force_enable_version(version);
        true
    }

    /// Disable the given version
    pub fn disable_version(&mut self, version: CamoVersion) {
        self.supported_versions[version.as_u8() as usize - 1] = false;
    }

    /// Returns whether or not the given version is supported by the `camo_` account **but** not necessarily supported by this software
    pub fn signals_version(&self, version: CamoVersion) -> bool {
        if !is_possible_version(version.as_u8()) {
            return false;
        }
        self.supported_versions[version.as_u8() as usize - 1]
    }

    /// Returns whether or not the given version is supported by the `camo_` account **and** supported by this software
    pub fn supports_version(&self, version: CamoVersion) -> bool {
        if !is_supported_version(version.as_u8()) {
            return false;
        }
        self.signals_version(version)
    }

    /// Returns the highest version that is supported by the `camo_` account **but** not necessarily supported by this software
    pub fn highest_signaled_version(&self) -> Option<CamoVersion> {
        ALL_POSSIBLE_CAMO_VERSIONS
            .iter()
            .rev()
            .find(|&&version| self.signals_version(version))
            .copied()
    }

    /// Returns the highest version that is supported by the `camo_` account **and** supported by this software
    pub fn highest_supported_version(&self) -> Option<CamoVersion> {
        ALL_POSSIBLE_CAMO_VERSIONS
            .iter()
            .rev()
            .find(|&&version| self.supports_version(version))
            .copied()
    }

    /// Returns all versions that are supported by the `camo_` account **but** not necessarily supported by this software
    pub fn all_signaled_versions(&self) -> Vec<CamoVersion> {
        ALL_POSSIBLE_CAMO_VERSIONS
            .iter()
            .filter(|&&version| self.signals_version(version))
            .copied()
            .collect()
    }

    /// Returns all versions that are supported by the `camo_` account **and** supported by this software
    pub fn all_supported_versions(&self) -> Vec<CamoVersion> {
        ALL_POSSIBLE_CAMO_VERSIONS
            .iter()
            .filter(|&&version| self.supports_version(version))
            .copied()
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
    use crate::constants::HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION;

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
        assert!(!is_possible_version(0.try_into().unwrap()));
        assert!(!is_supported_version(0.try_into().unwrap()));

        for i in super::ALL_POSSIBLE_CAMO_VERSIONS {
            assert!(is_possible_version(i.as_u8()));
        }
        for i in super::ALL_SUPPORTED_CAMO_VERSIONS {
            assert!(is_supported_version(i.as_u8()));
        }

        assert!(!is_possible_version(9.try_into().unwrap()));
        assert!(!is_supported_version(9.try_into().unwrap()));
    }

    #[test]
    fn highest_signaled_version() {
        assert!(TEST_VERSIONS_1.highest_signaled_version() == Some(6.try_into().unwrap()));
        assert!(TEST_VERSIONS_2.highest_signaled_version() == Some(8.try_into().unwrap()));
        assert!(TEST_VERSIONS_3.highest_signaled_version() == Some(8.try_into().unwrap()));
    }

    #[test]
    fn highest_supported_version() {
        assert!(TEST_VERSIONS_1.highest_supported_version() == Some(1.try_into().unwrap()));
        assert!(TEST_VERSIONS_2.highest_supported_version().is_none());
        assert!(TEST_VERSIONS_3.highest_supported_version() == Some(1.try_into().unwrap()));
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
#[cfg(test)]
#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use crate::serde_test;

    #[test]
    fn camo_version() {
        let bytes = bincode::serialize(&CamoVersion::Two).unwrap();
        assert!(bytes.len() == 1);
        let version: CamoVersion = bincode::deserialize(&bytes).unwrap();
        assert!(CamoVersion::Two == version);
    }
    serde_test!(camo_versions: CamoVersions::new_signaling(&[CamoVersion::Two, CamoVersion::Seven]) => 1);
}
