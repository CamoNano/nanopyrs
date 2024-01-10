use std::fmt::Display;
use std::error::Error;

#[cfg(feature = "stealth")]
use crate::stealth::StealthAccountVersions;

#[derive(Debug)]
pub enum NanoError {
    /// Invalid address length
    InvalidAddressLength,
    /// Invalid address prefix
    InvalidAddressPrefix,
    /// Invalid address checksum
    InvalidAddressChecksum,
    /// Invalid curve point
    InvalidCurvePoint,
    /// Invalid base32 encoding
    InvalidBase32,
    #[cfg(feature = "stealth")]
    /// unknown stealth protocol versions
    UnknownVersions(StealthAccountVersions)
}
impl Display for NanoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match &self {
            NanoError::InvalidAddressLength => "invalid length".into(),
            NanoError::InvalidAddressPrefix => "invalid formatting".into(),
            NanoError::InvalidBase32 => "invalid base 32 encoding".into(),
            NanoError::InvalidAddressChecksum => "invalid checksum".into(),
            NanoError::InvalidCurvePoint => "invalid ed25519 point".into(),
            #[cfg(feature = "stealth")]
            NanoError::UnknownVersions(versions) => format!("unknown stealth protocol versions: {versions:?}")
        };
        write!(f, "{string}")
    }
}
impl Error for NanoError {}