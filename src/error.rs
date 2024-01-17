use std::fmt::Display;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// incompatible stealth protocol versions
    #[cfg(feature = "stealth")]
    IncompatibleStealthVersions
}
impl Display for NanoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match &self {
            NanoError::InvalidAddressLength => "invalid length",
            NanoError::InvalidAddressPrefix => "invalid formatting",
            NanoError::InvalidBase32 => "invalid base 32 encoding",
            NanoError::InvalidAddressChecksum => "invalid checksum",
            NanoError::InvalidCurvePoint => "invalid ed25519 point",
            #[cfg(feature = "stealth")]
            NanoError::IncompatibleStealthVersions => "incompatible stealth protocol versions"
        }.into();
        write!(f, "{string}")
    }
}
impl Error for NanoError {}