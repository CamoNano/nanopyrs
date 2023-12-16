use super::{NanoError, try_point_from_slice};
use zeroize::Zeroize;
use curve25519_dalek::{EdwardsPoint, Scalar as RawScalar};

pub use super::nanopy::{sign_message, is_valid_signature};
pub mod hazmat {
    pub use crate::core::nanopy::sign_message_with_r;
}

#[derive(Debug, Clone, Copy, Zeroize)]
pub struct Signature {
    pub r: EdwardsPoint,
    pub s: RawScalar
}
impl Signature {
    pub fn to_bytes(&self) -> [u8; 64] {
        (*self).into()
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