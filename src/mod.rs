mod nanopy;
mod error;
mod secrets;
mod account;

pub mod constants;
pub mod hashes;
pub mod base32;
pub mod signature;
pub mod block;

pub use error::NanoError;
pub use secrets::{SecretBytes, Scalar};
pub use account::{Key, Account};
pub use signature::Signature;
pub use block::{Block, BlockType};


use curve25519_dalek::edwards::{EdwardsPoint, CompressedEdwardsY};

pub(crate) fn try_compressed_from_slice(key: &[u8]) -> Result<CompressedEdwardsY, NanoError> {
    CompressedEdwardsY::from_slice(key)
        .or( Err(NanoError::InvalidPoint) )
}

pub(crate) fn try_point_from_slice(key: &[u8]) -> Result<EdwardsPoint, NanoError> {
    try_compressed_from_slice(key)?
        .decompress().ok_or(NanoError::InvalidPoint)
}