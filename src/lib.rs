#![deny(unsafe_code)]

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

#[cfg(feature = "stealth")]
pub mod stealth;

#[cfg(feature = "rpc")]
pub mod rpc;



use curve25519_dalek::edwards::{EdwardsPoint, CompressedEdwardsY};

pub(crate) fn try_compressed_from_slice(key: &[u8]) -> Result<CompressedEdwardsY, NanoError> {
    CompressedEdwardsY::from_slice(key)
        .or( Err(NanoError::InvalidPoint) )
}

pub(crate) fn try_point_from_slice(key: &[u8]) -> Result<EdwardsPoint, NanoError> {
    let point = try_compressed_from_slice(key)?
        .decompress().ok_or(NanoError::InvalidPoint)?;
    if point.is_small_order() {
        return Err(NanoError::InvalidPoint)
    }
    Ok(point)
}

macro_rules! auto_from_impl {
    (TryFrom, $from: ty, $to: ty) => {
        impl TryFrom<$from> for $to {
            type Error = NanoError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                (&value).try_into()
            }
        }
    };

    (From, $from: ty, $to: ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                (&value).into()
            }
        }
    };
}
pub(crate) use auto_from_impl;