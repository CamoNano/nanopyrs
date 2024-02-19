use super::nanopy::{
    account_decode, account_encode, get_account_scalar, is_valid_signature, sign_message,
};
use super::{Block, Scalar, SecretBytes, Signature};
use crate::auto_from_impl;
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::{CompressedEdwardsY, EdwardsPoint},
    Scalar as RawScalar,
};
use std::fmt::Display;
use std::hash::Hash;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use super::error::NanoError;

#[cfg(feature = "rpc")]
use serde_json::Value as JsonValue;

/// The private key of a `nano_` account
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    private: Scalar,
}
impl Key {
    /// Get key at index (`i`) given 32-byte seed (`seed`)
    pub fn from_seed(seed: &SecretBytes<32>, i: u32) -> Key {
        Key {
            private: get_account_scalar(seed, i),
        }
    }

    pub fn from_scalar(scalar: Scalar) -> Key {
        Key::from(scalar)
    }

    pub fn as_scalar(&self) -> &Scalar {
        &self.private
    }

    pub fn to_account(&self) -> Account {
        Account::from(self)
    }

    /// Sign the `message` with this key, returning a `Signature`
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        sign_message(message, self)
    }

    /// Sign the `block` with this key, returning a `Signature`
    pub fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }
}
impl From<Scalar> for Key {
    fn from(value: Scalar) -> Self {
        Key { private: value }
    }
}
impl From<RawScalar> for Key {
    fn from(value: RawScalar) -> Self {
        Key::from(Scalar::from(value))
    }
}

impl_op_ex!(+ |a: &Key, b: &Key| -> Key {
    Key::from(&a.private + &b.private)
});
impl_op_ex!(-|a: &Key, b: &Key| -> Key { Key::from(&a.private - &b.private) });

impl_op_ex_commutative!(*|a: &Key, b: &EdwardsPoint| -> Account { Account::from(&a.private * b) });

/// A `nano_` account
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct Account {
    pub account: String,
    pub compressed: CompressedEdwardsY,
    pub point: EdwardsPoint,
}
impl Account {
    pub fn from_key(key: Key) -> Account {
        Account::from(key)
    }

    pub fn from_point(point: &EdwardsPoint) -> Account {
        Account::from(point)
    }

    pub fn from_compressed(compressed: &CompressedEdwardsY) -> Result<Account, NanoError> {
        Account::try_from(compressed)
    }

    #[cfg(feature = "stealth")]
    pub(crate) fn from_both_points(
        point: &EdwardsPoint,
        compressed: &CompressedEdwardsY,
    ) -> Account {
        Account {
            account: account_encode(compressed),
            compressed: *compressed,
            point: *point,
        }
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Result<Account, NanoError> {
        Account::try_from(bytes)
    }

    pub fn is_valid(account: &str) -> bool {
        Account::try_from(account).is_ok()
    }

    /// Check the validity of a signature made by this account's private key
    pub fn is_valid_signature(&self, message: &[u8], signature: &Signature) -> bool {
        is_valid_signature(message, signature, self)
    }
}
#[cfg(feature = "serde")]
impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.point.serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Account {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Account::from(EdwardsPoint::deserialize(deserializer)?))
    }
}

auto_from_impl!(FromStr: Account);
auto_from_impl!(From: Account => String);
auto_from_impl!(From: Key => Account);
auto_from_impl!(From: EdwardsPoint => Account);
auto_from_impl!(TryFrom: String => Account);
auto_from_impl!(TryFrom: CompressedEdwardsY => Account);
auto_from_impl!(TryFrom: [u8; 32] => Account);
#[cfg(feature = "rpc")]
auto_from_impl!(From: Account => JsonValue);

impl From<&Key> for Account {
    fn from(value: &Key) -> Self {
        value * G
    }
}
impl From<&EdwardsPoint> for Account {
    fn from(value: &EdwardsPoint) -> Self {
        let compressed = value.compress();
        let account = account_encode(&compressed);
        Account {
            account,
            compressed,
            point: *value,
        }
    }
}
impl TryFrom<&String> for Account {
    type Error = NanoError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Account::try_from(value as &str)
    }
}
impl TryFrom<&str> for Account {
    type Error = NanoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let compressed = account_decode(value)?;
        let point = compressed
            .decompress()
            .ok_or(NanoError::InvalidCurvePoint)?;
        Ok(Account {
            account: value.to_string(),
            compressed,
            point,
        })
    }
}
impl TryFrom<&CompressedEdwardsY> for Account {
    type Error = NanoError;
    fn try_from(value: &CompressedEdwardsY) -> Result<Self, Self::Error> {
        let account = account_encode(value);
        let point = value.decompress().ok_or(NanoError::InvalidCurvePoint)?;
        Ok(Account {
            account,
            compressed: *value,
            point,
        })
    }
}
impl TryFrom<&[u8; 32]> for Account {
    type Error = NanoError;
    fn try_from(value: &[u8; 32]) -> Result<Self, Self::Error> {
        let compressed =
            CompressedEdwardsY::from_slice(value).or(Err(NanoError::InvalidCurvePoint))?;
        Account::try_from(compressed)
    }
}
impl From<&Account> for String {
    fn from(val: &Account) -> Self {
        val.to_string()
    }
}
#[cfg(feature = "rpc")]
impl From<&Account> for JsonValue {
    fn from(val: &Account) -> Self {
        val.to_string().into()
    }
}
impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.account)
    }
}
impl Hash for Account {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.account.hash(state)
    }
}

impl_op_ex!(+ |a: &Account, b: &Account| -> Account {
    Account::from(a.point + b.point)
});
impl_op_ex!(-|a: &Account, b: &Account| -> Account { Account::from(a.point - b.point) });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::get_genesis_account, SecretBytes};

    #[test]
    fn from_str() {
        let genesis = get_genesis_account().to_string();
        let genesis = genesis.parse::<Account>().unwrap();
        assert!(genesis == get_genesis_account());

        let seed = SecretBytes::from([0; 32]);
        let account = Key::from_seed(&seed, 0).to_account();
        assert!(
            account.to_string()
                == "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );
    }

    #[test]
    fn math() {
        let seed = SecretBytes::from([0; 32]);

        let key_1 = Key::from_seed(&seed, 0);
        let key_2 = Key::from_seed(&seed, 1);
        let account_1 = key_1.to_account();
        let account_2 = key_2.to_account();
        assert!((key_1 + key_2).to_account() == account_1 + account_2)
    }
}
