use std::fmt::Display;
use std::error::Error;

#[cfg(feature = "stealth")]
use crate::stealth::StealthAccountVersions;

#[derive(Debug)]
pub enum NanoError {
    InvalidLength,
    InvalidFormatting,
    InvalidBase32,
    InvalidChecksum,
    InvalidPoint,
    #[cfg(feature = "stealth")]
    InvalidVersions(StealthAccountVersions)
}
impl Display for NanoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match &self {
            NanoError::InvalidLength => "invalid length".into(),
            NanoError::InvalidFormatting => "invalid formatting".into(),
            NanoError::InvalidBase32 => "invalid base 32 encoding".into(),
            NanoError::InvalidChecksum => "invalid checksum".into(),
            NanoError::InvalidPoint => "invalid ed25519 point".into(),
            #[cfg(feature = "stealth")]
            NanoError::InvalidVersions(bits) => format!("invalid version bits: {bits}")
        };
        write!(f, "{string}")
    }
}
impl Error for NanoError {}