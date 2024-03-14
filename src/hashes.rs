use crate::{scalar, secret, Scalar, SecretBytes};
use blake2::{
    digest::consts::{U32, U5, U64, U8},
    Blake2b as _Blake2b, Digest,
};
use curve25519_dalek::scalar::{clamp_integer, Scalar as RawScalar};

#[cfg(feature = "camo")]
use crate::constants::{SPEND_CONSTANTS_X_INDEX, VIEW_CONSTANTS_X_INDEX};

pub mod hazmat {
    pub use crate::nanopy::{get_account_scalar, get_account_seed};

    #[cfg(feature = "camo")]
    use super::*;
    #[cfg(feature = "camo")]
    pub fn get_category_seed(seed: &SecretBytes<32>, i: u32) -> SecretBytes<32> {
        blake2b256(&[&i.to_be_bytes(), seed.as_slice()].concat())
    }
}
#[cfg(feature = "camo")]
use hazmat::get_category_seed;

/// Returns the wallet's master spend seed.
///
/// Equivalent to `hazmat::get_category_seed(seed, SPEND_CONSTANTS_X_INDEX)`
#[cfg(feature = "camo")]
pub fn get_camo_spend_seed(master_seed: &SecretBytes<32>) -> SecretBytes<32> {
    get_category_seed(master_seed, SPEND_CONSTANTS_X_INDEX)
}

/// Returns the wallet's master view seed.
///
/// Equivalent to `hazmat::get_category_seed(seed, VIEW_CONSTANTS_X_INDEX)`\
#[cfg(feature = "camo")]
pub fn get_camo_view_seed(master_seed: &SecretBytes<32>) -> SecretBytes<32> {
    get_category_seed(master_seed, VIEW_CONSTANTS_X_INDEX)
}

type Blake2b512 = _Blake2b<U64>;
type Blake2b256 = _Blake2b<U32>;
type Blake2bWork = _Blake2b<U8>;
type Blake2bChecksum = _Blake2b<U5>;

pub fn blake2b512(input: &[u8]) -> SecretBytes<64> {
    let mut hasher = Blake2b512::new();
    hasher.update(input);
    let hash: [u8; 64] = hasher.finalize().into();
    secret!(hash)
}

pub fn blake2b256(input: &[u8]) -> SecretBytes<32> {
    let mut hasher = Blake2b256::new();
    hasher.update(input);
    let hash: [u8; 32] = hasher.finalize().into();
    secret!(hash)
}

pub fn blake2b_work(input: &[u8]) -> [u8; 8] {
    let mut hasher = Blake2bWork::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn blake2b_checksum(input: &[u8]) -> [u8; 5] {
    let mut hasher = Blake2bChecksum::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn blake2b_scalar(input: &[u8]) -> Scalar {
    scalar!(RawScalar::from_bytes_mod_order(clamp_integer(
        blake2b512(input).as_ref()[..32].try_into().unwrap()
    )))
}

#[cfg(test)]
mod tests {
    use crate::Scalar;

    #[test]
    fn blake2b512() {
        let result = super::blake2b512(b"test");
        assert!(result.as_ref()[..5] == [167, 16, 121, 212, 40])
    }
    #[test]
    fn blake2b256() {
        let result = super::blake2b256(b"test");
        assert!(result.as_ref()[..5] == [146, 139, 32, 54, 105])
    }
    #[test]
    fn blake2b_work() {
        let result = super::blake2b_work(b"test");
        assert!(result.as_ref()[..5] == [150, 173, 59, 180, 162])
    }
    #[test]
    fn blake2b_checksum() {
        let result = super::blake2b_checksum(b"test");
        assert!(result.as_ref()[..5] == [210, 40, 235, 33, 186])
    }
    #[test]
    fn blake2b_scalar() {
        let bytes: [u8; 32] = *b"\xa7\x10y\xd4(S\xde\xa2nE0\x043\x86p\xa58\x14\xb7\x817\xff\xbe\xd0v\x03\xa4\x1dv\xa4\x83\xaa";
        let result = super::blake2b_scalar(b"test");
        assert!(result == Scalar::from(bytes))
    }
}
