use super::{NanoError, Key, Account, try_point_from_slice};
use zeroize::Zeroize;
use curve25519_dalek::{EdwardsPoint, Scalar as RawScalar};

pub use super::nanopy::{sign_message, is_valid_signature};
pub mod hazmat {
    pub use crate::core::nanopy::sign_message_with_r;
}

#[derive(Debug, Clone, Copy, Zeroize, PartialEq, Eq)]
pub struct Signature {
    pub r: EdwardsPoint,
    pub s: RawScalar
}
impl Signature {
    pub fn to_bytes(&self) -> [u8; 64] {
        (*self).into()
    }

    pub fn new(message: &[u8], key: &Key) -> Signature {
        key.sign_message(message)
    }

    pub fn is_valid(&self, message: &[u8], account: &Account) -> bool {
        account.is_valid_signature(message, *self)
    }
}
impl From<Signature> for [u8; 64] {
    fn from(value: Signature) -> Self {
        [
            value.r.compress().to_bytes(),
            value.s.to_bytes()
        ].concat().try_into().unwrap()
    }
}
impl TryFrom<[u8; 64]> for Signature {
    type Error = NanoError;

    fn try_from(value: [u8; 64]) -> Result<Self, NanoError> {
        let r = try_point_from_slice(&value[..32])?;
        let s = RawScalar::from_bytes_mod_order(
            value[32..64].try_into().unwrap()
        );
        Ok(Signature{r, s})
    }
}

#[cfg(test)]
mod tests {
    use crate::{SecretBytes, Key};

    fn get_key(seed: [u8; 32], i: u32) -> Key {
        let seed = SecretBytes::from(&mut seed.clone());
        Key::from_seed(&seed, i)
    }

    #[test]
    fn valid() {
        let key = get_key([0; 32], 0);
        let account = key.to_account();
        let signature = key.sign_message(b"test");
        assert!(account.is_valid_signature(b"test", signature))
    }

    #[test]
    fn invalid_key() {
        let key = get_key([0; 32], 0);
        let account = get_key([0; 32], 1).to_account();
        let signature = key.sign_message(b"test");
        assert!(!account.is_valid_signature(b"test", signature))
    }

    #[test]
    fn invalid_message() {
        let key = get_key([0; 32], 0);
        let account = key.to_account();
        let signature = key.sign_message(b"test 1");
        assert!(!account.is_valid_signature(b"test 2", signature))
    }

    #[test]
    fn r_safety() {
        let key = get_key([0; 32], 0);
        let signature_1 = key.sign_message(b"test 1");
        let signature_2 = key.sign_message(b"test 2");
        assert!(signature_1 != signature_2);
        assert!(signature_1.r != signature_2.r);
        assert!(signature_1.s != signature_2.s);
    }
}