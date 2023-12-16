use crate::{secret, scalar};
use super::{SecretBytes, Scalar};
use curve25519_dalek::scalar::{
    Scalar as RawScalar,
    clamp_integer
};
use blake2::{
    Blake2b as _Blake2b,
    Digest,
    digest::consts::{U64, U32, U8, U5}
};

type Blake2b512 = _Blake2b<U64>;
type Blake2b256 = _Blake2b<U32>;
type Blake2bWork = _Blake2b<U8>;
type Blake2bChecksum = _Blake2b<U5>;

pub fn blake2b512(input: &[u8]) -> SecretBytes<64> {
    let mut hasher = Blake2b512::new();
    hasher.update(input);
    secret!(&mut hasher.finalize().into())
}

pub fn blake2b256(input: &[u8]) -> SecretBytes<32> {
    let mut hasher = Blake2b256::new();
    hasher.update(input);
    secret!(&mut hasher.finalize().into())
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
    scalar!(&mut RawScalar::from_bytes_mod_order(clamp_integer(
        blake2b512(input).as_ref()[..32].try_into().unwrap()
    )))
}

pub fn get_account_seed(seed: &SecretBytes<32>, i: u32) -> SecretBytes<32> {
    blake2b256(&[seed.as_slice(), &i.to_be_bytes()].concat())
}