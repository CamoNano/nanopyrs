use crate::auto_from_impl;
use super::nanopy::{
    get_account_scalar,
    account_encode, account_decode,
    sign_message, is_valid_signature
};
use super::{SecretBytes, Scalar, Block, Signature};
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use std::fmt::Display;
use std::hash::Hash;
use zeroize::{Zeroize, ZeroizeOnDrop};
use curve25519_dalek::{
    Scalar as RawScalar,
    edwards::{EdwardsPoint, CompressedEdwardsY},
    constants::ED25519_BASEPOINT_POINT as G
};

pub use super::error::NanoError;

#[cfg(feature = "rpc")]
use serde_json::Value as JsonValue;



#[derive(Debug, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct Key {
    private: Scalar
}
impl Key {
    /// Get key at index (`i`) given 32-byte seed (`seed`)
    pub fn from_seed(seed: &SecretBytes<32>, i: u32) -> Key {
        Key { private: get_account_scalar(seed, i) }
    }

    pub fn from_scalar(scalar: Scalar) -> Key {
        Key::from(scalar)
    }

    pub fn to_account(&self) -> Account {
        Account::from(self)
    }

    pub fn as_scalar(&self) -> &Scalar {
        &self.private
    }

    pub fn sign_message(&self, message: &[u8]) -> Signature {
        sign_message(message, self)
    }

    pub fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }
}
impl From<Scalar> for Key {
    fn from(value: Scalar) -> Self {
        Key { private: value }
    }
}
impl From<&mut RawScalar> for Key {
    fn from(value: &mut RawScalar) -> Self {
        Key::from(Scalar::from(value))
    }
}

impl_op_ex!(+ |a: &Key, b: &Key| -> Key {
    Key::from(&a.private + &b.private)
});
impl_op_ex!(- |a: &Key, b: &Key| -> Key {
    Key::from(&a.private - &b.private)
});

impl_op_ex_commutative!(* |a: &Key, b: &EdwardsPoint| -> Account {
    Account::from(&a.private * b)
});



#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct Account {
    pub account: String,
    pub compressed: CompressedEdwardsY,
    pub point: EdwardsPoint
}
impl Account {
    pub fn from_key(key: Key) -> Account {
        Account::from(key)
    }

    pub fn from_point(point: &EdwardsPoint) -> Account {
        Account::from(point)
    }

    pub fn from_string(account: &str) -> Result<Account, NanoError> {
        Account::try_from(account)
    }

    pub fn from_compressed(compressed: &CompressedEdwardsY) -> Result<Account, NanoError> {
        Account::try_from(compressed)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Result<Account, NanoError> {
        Account::try_from(bytes)
    }

    pub fn is_valid(account: &str) -> bool {
        Account::try_from(account).is_ok()
    }

    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        is_valid_signature(message, signature, self)
    }
}

auto_from_impl!(From, Account, String);
#[cfg(feature = "rpc")]
auto_from_impl!(From, Account, JsonValue);
auto_from_impl!(From, Key, Account);
auto_from_impl!(From, EdwardsPoint, Account);
auto_from_impl!(TryFrom, String, Account);
auto_from_impl!(TryFrom, CompressedEdwardsY, Account);
auto_from_impl!(TryFrom, [u8; 32], Account);

impl From<&Key> for Account {
    fn from(value: &Key) -> Self {
        value * G
    }
}
impl From<&EdwardsPoint> for Account {
    fn from(value: &EdwardsPoint) -> Self {
        let compressed = value.compress();
        let account = account_encode(&compressed);
        Account{account, compressed, point: *value}
    }
}
impl TryFrom<&String> for Account {
    type Error = NanoError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let compressed = account_decode(&value)?;
        let point = compressed.decompress()
            .ok_or(NanoError::InvalidPoint)?;
        Ok(Account{account: value.to_string(), compressed, point})
    }
}
impl TryFrom<&str> for Account {
    type Error = NanoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Account::try_from(value.to_owned())
    }
}
impl TryFrom<&CompressedEdwardsY> for Account {
    type Error = NanoError;
    fn try_from(value: &CompressedEdwardsY) -> Result<Self, Self::Error> {
        let account = account_encode(&value);
        let point = value.decompress()
            .ok_or(NanoError::InvalidPoint)?;
        Ok(Account{account, compressed: *value, point})
    }
}
impl TryFrom<&[u8; 32]> for Account {
    type Error = NanoError;
    fn try_from(value: &[u8; 32]) -> Result<Self, Self::Error> {
        let compressed = CompressedEdwardsY::from_slice(value)
            .or(Err(NanoError::InvalidPoint))?;
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
        self.account.hash(state);
        self.compressed.hash(state);
    }
}

impl_op_ex!(+ |a: &Account, b: &Account| -> Account {
    Account::from(a.point + b.point)
});
impl_op_ex!(- |a: &Account, b: &Account| -> Account {
    Account::from(a.point - b.point)
});



#[cfg(test)]
mod tests {
    use crate::{SecretBytes, constants::get_genesis_account};
    use super::*;

    #[test]
    fn from_string() {
        let genesis = get_genesis_account().to_string();
        let genesis = Account::try_from(&genesis).unwrap();
        assert!(genesis == get_genesis_account());

        let seed = SecretBytes::from(&mut [0; 32]);
        let account = Key::from_seed(&seed, 0).to_account();
        assert!(account.to_string() == "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7");
    }

    #[test]
    fn math() {
        let seed = SecretBytes::from(&mut [0; 32]);

        let key_1 = Key::from_seed(&seed, 0);
        let key_2 = Key::from_seed(&seed, 1);
        let account_1 = key_1.to_account();
        let account_2 = key_2.to_account();
        assert!((key_1 + key_2).to_account() == account_1 + account_2)
    }
}