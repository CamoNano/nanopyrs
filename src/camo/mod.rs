mod v1;
mod version;

// pub mod hazmat {
//     pub mod v1 {
//         pub use crate::camo::v1::*;
//     }
// }

use crate::{
    auto_from_impl, base32,
    constants::{ADDRESS_CHARS_SAMPLE_END, CAMO_ACCOUNT_PREFIX, CAMO_PREFIX_LEN},
    version_bits, Account, Block, Key, NanoError, SecretBytes, Signature,
};
use curve25519_dalek::edwards::EdwardsPoint;
use std::fmt::Display;
use std::str::FromStr;
use v1::{CamoAccountV1, CamoKeysV1, CamoViewKeysV1};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use version::CamoVersions;

/// Based on the hash of the notification block,
/// return the "standard" index to derive the one-time account from using the shared seed.
pub fn get_standard_index(notification_block: &Block) -> u32 {
    u32::from_be_bytes(notification_block.hash()[..4].try_into().unwrap())
}

macro_rules! unwrap_enum {
    (CamoKeys, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            CamoKeys::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
    (CamoViewKeys, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            CamoViewKeys::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
    (CamoAccount, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            CamoAccount::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
}

pub(crate) trait CamoKeysTrait: Sized + Zeroize + PartialEq + Eq {
    type ViewKeysType: CamoViewKeysTrait;
    type AccountType: CamoAccountTrait;

    fn from_seed(seed: &SecretBytes<32>, i: u32, versions: CamoVersions) -> Self;
    fn to_view_keys(&self) -> Self::ViewKeysType;
    fn to_camo_account(&self) -> Self::AccountType;

    fn notification_key(&self) -> Key;
    fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }
    fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }

    fn get_versions(&self) -> CamoVersions;

    fn receiver_ecdh(&self, sender_account: &Account) -> SecretBytes<32>;
    fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key;
    fn derive_key(&self, sender_account: &Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }
    fn derive_key_from_block(&self, block: &Block) -> Key;
}

/// The private keys of a `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoKeys {
    V1(Box<CamoKeysV1>) = 1,
}
impl CamoKeys {
    /// Returns `None` if no supported version is given
    pub fn from_seed(seed: &SecretBytes<32>, i: u32, versions: CamoVersions) -> Option<CamoKeys> {
        match versions.highest_supported_version() {
            Some(1) => Some(CamoKeys::V1(Box::new(CamoKeysV1::from_seed(
                seed, i, versions,
            )))),
            _ => None,
        }
    }

    pub fn to_view_keys(&self) -> CamoViewKeys {
        self.into()
    }

    pub fn to_camo_account(&self) -> CamoAccount {
        self.into()
    }

    /// Key of the account for "notification" transactions to be sent to, if applicable
    pub fn notification_key(&self) -> Key {
        unwrap_enum!(CamoKeys, self.notification_key())
    }

    /// Sign the `message` with the notification key, returning a `Signature`
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }
    /// Sign the `block` with the notification key, returning a `Signature`
    pub fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }

    /// Get the versions which this `camo_` account supports
    pub fn get_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoKeys, self.get_versions())
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, sender_account: &Account) -> SecretBytes<32> {
        unwrap_enum!(CamoKeys, self.receiver_ecdh(sender_account))
    }

    pub fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        unwrap_enum!(CamoKeys, self.derive_key_from_secret(secret, i))
    }

    pub fn derive_key(&self, sender_account: &Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }

    /// Derive a one-time key from the notification block.
    ///
    /// Similar to `derive_key()`, except a psuedo-random index is automatically calculated.
    pub fn derive_key_from_block(&self, block: &Block) -> Key {
        unwrap_enum!(CamoKeys, self.derive_key_from_block(block))
    }
}

pub(crate) trait CamoViewKeysTrait: Sized + Zeroize + PartialEq + Eq {
    type AccountType: CamoAccountTrait;

    fn from_seed(
        view_seed: &SecretBytes<32>,
        master_spend: EdwardsPoint,
        i: u32,
        versions: CamoVersions,
    ) -> Self;
    fn to_camo_account(&self) -> Self::AccountType;

    fn notification_account(&self) -> Account;
    fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account()
            .is_valid_signature(message, &signature)
    }

    fn get_versions(&self) -> CamoVersions;

    fn receiver_ecdh(&self, sender_account: &Account) -> SecretBytes<32>;
    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account;
    fn derive_account(&self, sender_account: &Account, i: u32) -> Account {
        self.derive_account_from_secret(&self.receiver_ecdh(sender_account), i)
    }
    fn derive_account_from_block(&self, block: &Block) -> Account;
}

/// The private view keys of a `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoViewKeys {
    V1(Box<CamoViewKeysV1>) = 1,
}
impl CamoViewKeys {
    pub fn from_keys(keys: CamoKeys) -> CamoViewKeys {
        keys.to_view_keys()
    }

    /// Returns `None` if no supported version is given
    pub fn from_seed(
        seed: &SecretBytes<32>,
        master_spend: EdwardsPoint,
        i: u32,
        versions: CamoVersions,
    ) -> Option<CamoViewKeys> {
        match versions.highest_supported_version() {
            Some(1) => Some(CamoViewKeys::V1(Box::new(CamoViewKeysV1::from_seed(
                seed,
                master_spend,
                i,
                versions,
            )))),
            _ => None,
        }
    }

    pub fn to_camo_account(&self) -> CamoAccount {
        self.into()
    }

    pub fn to_bytes(&self) -> SecretBytes<65> {
        self.into()
    }

    pub fn from_bytes(value: &SecretBytes<65>) -> Option<CamoViewKeys> {
        CamoViewKeys::try_from(value).ok()
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(CamoViewKeys, self.notification_account())
    }

    /// Check the validity of a signature made by the notification key
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account()
            .is_valid_signature(message, &signature)
    }

    /// Get the versions which this `camo_` account supports
    pub fn get_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoViewKeys, self.get_versions())
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, sender_account: &Account) -> SecretBytes<32> {
        unwrap_enum!(CamoViewKeys, self.receiver_ecdh(sender_account))
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(CamoViewKeys, self.derive_account_from_secret(secret, i))
    }

    pub fn derive_account(&self, sender_account: &Account, i: u32) -> Account {
        self.derive_account_from_secret(&self.receiver_ecdh(sender_account), i)
    }

    /// Derive a one-time account from the notification block.
    ///
    /// Similar to `derive_account()`, except a psuedo-random index is automatically calculated.
    pub fn derive_account_from_block(&self, block: &Block) -> Account {
        unwrap_enum!(CamoViewKeys, self.derive_account_from_block(block))
    }
}

auto_from_impl!(From: CamoViewKeys => SecretBytes<65>);
auto_from_impl!(From: CamoKeys => CamoViewKeys);

impl From<&CamoViewKeys> for SecretBytes<65> {
    fn from(value: &CamoViewKeys) -> Self {
        unwrap_enum!(CamoViewKeys, value.into())
    }
}
impl TryFrom<SecretBytes<65>> for CamoViewKeys {
    type Error = ();

    fn try_from(value: SecretBytes<65>) -> Result<Self, ()> {
        (&value).try_into()
    }
}
impl TryFrom<&SecretBytes<65>> for CamoViewKeys {
    type Error = ();

    fn try_from(value: &SecretBytes<65>) -> Result<Self, ()> {
        let versions = CamoVersions::decode_from_bits(value.as_ref()[0]);

        let value = match CamoViewKeysV1::try_from(value) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        match versions.highest_supported_version() {
            Some(1) => Ok(CamoViewKeys::V1(Box::new(value))),
            _ => Err(()),
        }
    }
}
impl From<&CamoKeys> for CamoViewKeys {
    fn from(value: &CamoKeys) -> Self {
        match value {
            CamoKeys::V1(v1) => CamoViewKeys::V1(Box::new(v1.to_view_keys())),
        }
    }
}

pub(crate) trait CamoAccountTrait: Sized + Zeroize + Display + PartialEq + Eq {
    type KeysType: CamoKeysTrait;

    fn from_keys(keys: Self::KeysType) -> Self;
    fn from_data(account: &str, data: &[u8]) -> Result<Self, NanoError>;
    fn from_str(account: &str) -> Result<Self, NanoError> {
        let data = base32::decode(&account[CAMO_PREFIX_LEN..]).ok_or(NanoError::InvalidBase32)?;
        Self::from_data(account, &data)
    }

    fn notification_account(&self) -> Account;
    fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account()
            .is_valid_signature(message, &signature)
    }

    fn get_versions(&self) -> CamoVersions;
    fn is_valid(account: &str) -> bool {
        Self::from_str(account).is_ok()
    }

    fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32>;
    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account;
    fn derive_account(&self, sender_key: &Key, i: u32) -> Account {
        self.derive_account_from_secret(&self.sender_ecdh(sender_key), i)
    }
    fn derive_account_from_block(&self, block: &Block, sender_key: &Key) -> Account;
}

/// A `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoAccount {
    V1(Box<CamoAccountV1>) = 1,
}
impl CamoAccount {
    pub fn from_keys(keys: CamoKeys) -> CamoAccount {
        keys.to_camo_account()
    }

    pub fn from_view_keys(keys: CamoViewKeys) -> CamoAccount {
        keys.to_camo_account()
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(CamoAccount, self.notification_account())
    }

    /// Check the validity of a signature made by the notification key
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account()
            .is_valid_signature(message, &signature)
    }

    /// Get the versions which this `camo_` account supports
    pub fn get_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoAccount, self.get_versions())
    }

    pub fn is_valid(account: &str) -> bool {
        Self::from_str(account).is_ok()
    }

    /// Calculate the shared secret between this account and the given key.
    pub fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32> {
        unwrap_enum!(CamoAccount, self.sender_ecdh(sender_key))
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(CamoAccount, self.derive_account_from_secret(secret, i))
    }

    pub fn derive_account(&self, sender_key: &Key, i: u32) -> Account {
        self.derive_account_from_secret(&self.sender_ecdh(sender_key), i)
    }

    /// Derive a one-time account from the notification block.
    ///
    /// Similar to `derive_account()`, except a psuedo-random index is automatically calculated.
    pub fn derive_account_from_block(&self, block: &Block, sender_key: &Key) -> Account {
        unwrap_enum!(
            CamoAccount,
            self.derive_account_from_block(block, sender_key)
        )
    }
}
impl FromStr for CamoAccount {
    type Err = NanoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // sanity check to prevent panic
        if s.len() < ADDRESS_CHARS_SAMPLE_END {
            return Err(NanoError::InvalidAddressLength);
        }
        if &s[..CAMO_PREFIX_LEN] != CAMO_ACCOUNT_PREFIX {
            return Err(NanoError::InvalidAddressPrefix);
        }
        let address_sample = &s[CAMO_PREFIX_LEN..ADDRESS_CHARS_SAMPLE_END];
        let data = base32::decode(address_sample).ok_or(NanoError::InvalidBase32)?;

        match version_bits!(data[0]).highest_supported_version() {
            Some(1) => Ok(CamoAccount::V1(Box::new(CamoAccountV1::from_str(s)?))),
            _ => Err(NanoError::IncompatibleCamoVersions),
        }
    }
}

auto_from_impl!(From: CamoKeys => CamoAccount);
auto_from_impl!(From: CamoViewKeys => CamoAccount);

impl From<&CamoKeys> for CamoAccount {
    fn from(value: &CamoKeys) -> Self {
        match value {
            CamoKeys::V1(v1) => CamoAccount::V1(Box::new(v1.to_camo_account())),
        }
    }
}
impl From<&CamoViewKeys> for CamoAccount {
    fn from(value: &CamoViewKeys) -> Self {
        match value {
            CamoViewKeys::V1(v1) => CamoAccount::V1(Box::new(v1.to_camo_account())),
        }
    }
}
impl Display for CamoAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = unwrap_enum!(CamoAccount, self.to_string());
        write!(f, "{}", as_string)
    }
}

pub(super) trait AutoTestUtils: Sized {
    fn unwrap(self) -> Self {
        self
    }
}

macro_rules! camo_address_tests {
    ($keys: ident, $view_keys: ident, $account: ident, $versions: expr, $addr: expr) => {
        impl AutoTestUtils for $keys {}
        impl AutoTestUtils for $account {}

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::versions;

            #[test]
            fn camo_account() {
                let seed = SecretBytes::from([0; 32]);
                let key = $keys::from_seed(&seed, 0, $versions).unwrap();
                let view_keys = key.to_view_keys();
                let account = key.to_camo_account();

                assert!(account.to_string() == $addr);
                assert!(account == $account::from_str($addr).unwrap());

                assert!($versions == key.get_versions());
                assert!($versions == view_keys.get_versions());
                assert!($versions == account.get_versions());
            }

            #[test]
            fn notification_account() {
                let seed = SecretBytes::from([0; 32]);
                let keys = $keys::from_seed(&seed, 0, $versions).unwrap();
                let view_keys = keys.to_view_keys();
                let account = keys.to_view_keys();

                let keys_account = keys.notification_key().to_account();
                let view_keys_account = view_keys.notification_account();
                let public_account = account.notification_account();

                assert!(keys_account == view_keys_account);
                assert!(keys_account == public_account);
            }

            #[test]
            fn derive_account() {
                let seed = SecretBytes::from([127; 32]);

                let sender_keys = Key::from_seed(&seed, 0);
                let sender_account = sender_keys.to_account();

                let recipient_keys = $keys::from_seed(&seed, 99, $versions).unwrap();
                let recipient_view_keys = recipient_keys.to_view_keys();
                let recipient_account = recipient_keys.to_camo_account();

                let recipient_derived = recipient_keys.derive_key(&sender_account, 0).to_account();
                let recipient_vk_derived = recipient_view_keys.derive_account(&sender_account, 0);
                let sender_derived = recipient_account.derive_account(&sender_keys, 0);

                assert!(recipient_derived == recipient_vk_derived);
                assert!(recipient_derived == sender_derived);
            }

            #[test]
            fn view_keys_bytes() {
                let seed = SecretBytes::from([42; 32]);
                let sender_view_keys_1 = $keys::from_seed(&seed, 99, $versions)
                    .unwrap()
                    .to_view_keys();

                let bytes: SecretBytes<65> = (&sender_view_keys_1).into();
                let sender_view_keys_2 = $view_keys::try_from(bytes).unwrap().into();

                assert!(sender_view_keys_1 == sender_view_keys_2);
            }
        }
    };
}
pub(crate) use camo_address_tests;

#[cfg(test)]
use crate::constants::HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION;
camo_address_tests!(
    CamoKeys, CamoViewKeys, CamoAccount,
    versions!(HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION),
    "camo_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);