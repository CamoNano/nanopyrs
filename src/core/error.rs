use std::fmt::Display;
use std::error::Error;

#[derive(Debug)]
pub enum NanoError {
    InvalidLength,
    InvalidFormatting,
    InvalidBase32,
    InvalidChecksum,
    InvalidPoint
}
impl Display for NanoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match &self {
            NanoError::InvalidLength => "invalid length",
            NanoError::InvalidFormatting => "invalid formatting",
            NanoError::InvalidBase32 => "invalid base 32 encoding",
            NanoError::InvalidChecksum => "invalid checksum",
            NanoError::InvalidPoint => "invalid ed25519 point"
        };
        write!(f, "{string}")
    }
}
impl Error for NanoError {}