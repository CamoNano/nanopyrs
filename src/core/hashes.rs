use crate::{secret, scalar, SecretBytes, Scalar};
use curve25519_dalek::scalar::{
    Scalar as RawScalar,
    clamp_integer
};
use blake2::{
    Blake2b as _Blake2b,
    Digest,
    digest::consts::{U64, U32, U8, U5}
};

pub mod hazmat {
    pub use crate::core::nanopy::{get_account_seed, get_account_private_key};
}

type Blake2b512 = _Blake2b<U64>;
type Blake2b256 = _Blake2b<U32>;
type Blake2bWork = _Blake2b<U8>;
type Blake2bChecksum = _Blake2b<U5>;

macro_rules! hash {
    ($type: ty, $input: expr) => {
        {
            let mut hasher = <$type>::new();
            hasher.update($input);
            hasher.finalize().into()
        }
    };
}

pub fn blake2b512(input: &[u8]) -> SecretBytes<64> {
    secret!(&mut hash!(Blake2b512, input))
}

pub fn blake2b256(input: &[u8]) -> SecretBytes<32> {
    secret!(&mut hash!(Blake2b256, input))
}

pub fn blake2b_work(input: &[u8]) -> [u8; 8] {
    hash!(Blake2bWork, input)
}

pub fn blake2b_checksum(input: &[u8]) -> [u8; 5] {
    hash!(Blake2bChecksum, input)
}

pub fn blake2b_scalar(input: &[u8]) -> Scalar {
    scalar!(&mut RawScalar::from_bytes_mod_order(clamp_integer(
        blake2b512(input).as_ref()[..32].try_into().unwrap()
    )))
}

#[cfg(test)]
mod tests {
    #[test]
    fn blake2b512() {
        let result = super::blake2b512(b"test");
        assert!(&result.as_ref()[..5] == &[167, 16, 121, 212, 40])
    }
    #[test]
    fn blake2b256() {
        let result = super::blake2b256(b"test");
        assert!(&result.as_ref()[..5] == &[146, 139, 32, 54, 105])
    }
    #[test]
    fn blake2b_work() {
        let result = super::blake2b_work(b"test");
        assert!(&result.as_ref()[..5] == &[150, 173, 59, 180, 162])
    }
    #[test]
    fn blake2b_checksum() {
        let result = super::blake2b_checksum(b"test");
        assert!(&result.as_ref()[..5] == &[210, 40, 235, 33, 186])
    }
    // blake2b_scalar is covered by blake2b512
}