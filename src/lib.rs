#![warn(unused_crate_dependencies, unsafe_code)]

mod account;
mod error;
mod nanopy;
mod secrets;

pub mod base32;
pub mod block;
/// Various Nano-related constants
pub mod constants;
/// Various hash functions
pub mod hashes;
pub mod signature;

pub use account::{Account, Key};
pub use block::{Block, BlockType};
pub use error::NanoError;
pub use secrets::{Scalar, SecretBytes};
pub use signature::Signature;

#[cfg(feature = "stealth")]
pub mod stealth;

#[cfg(feature = "rpc")]
pub mod rpc;

use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};

pub(crate) fn try_compressed_from_slice(key: &[u8]) -> Result<CompressedEdwardsY, NanoError> {
    CompressedEdwardsY::from_slice(key).or(Err(NanoError::InvalidCurvePoint))
}

pub(crate) fn try_point_from_slice(key: &[u8]) -> Result<EdwardsPoint, NanoError> {
    let point = try_compressed_from_slice(key)?
        .decompress()
        .ok_or(NanoError::InvalidCurvePoint)?;
    if point.is_small_order() {
        return Err(NanoError::InvalidCurvePoint);
    }
    Ok(point)
}

macro_rules! auto_from_impl {
    (TryFrom: $from: ty => $to: ty) => {
        impl TryFrom<$from> for $to {
            type Error = NanoError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                (&value).try_into()
            }
        }
    };

    (From: $from: ty => $to: ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                (&value).into()
            }
        }
    };

    (FromStr: $from: ty) => {
        use std::str::FromStr;
        impl FromStr for $from {
            type Err = NanoError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$from>::try_from(s)
            }
        }
    };
}
pub(crate) use auto_from_impl;
