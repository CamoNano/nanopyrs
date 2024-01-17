mod version;
mod v1;

// pub mod hazmat {
//     pub mod v1 {
//         pub use crate::stealth::v1::*;
//     }
// }

use crate::{
    auto_from_impl,
    version_bits,
    base32,
    NanoError, Key, Account, Block, Signature, SecretBytes,
    constants::{STEALTH_ACCOUNT_PREFIX, STEALTH_PREFIX_LEN, ADDRESS_CHARS_SAMPLE_END}
};
use v1::{StealthKeysV1, StealthViewKeysV1, StealthAccountV1};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};
use curve25519_dalek::edwards::EdwardsPoint;

pub use version::StealthAccountVersions;

macro_rules! unwrap_enum {
    (StealthKeys, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            StealthKeys::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
    (StealthViewKeys, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            StealthViewKeys::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
    (StealthAccount, $instance:ident . $func:ident($($arg:expr),*) ) => {
        match $instance {
            StealthAccount::V1(v1) => v1.as_ref().$func($($arg),*)
        }
    };
}



pub(crate) trait StealthKeysTrait: Sized + Zeroize + ZeroizeOnDrop  {
    type ViewKeysType: StealthViewKeysTrait;
    type AccountType: StealthAccountTrait;

    fn from_seed(seed: &SecretBytes<32>, i: u32, versions: StealthAccountVersions) -> Self;
    fn to_view_keys(&self) -> Self::ViewKeysType;
    fn to_stealth_account(&self) -> Self::AccountType;

    fn notification_key(&self) -> Key;
    fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }
    fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }

    fn get_versions(&self) -> StealthAccountVersions;

    fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32>;
    fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key;
    fn derive_key(&self, sender_account: Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}

/// The private keys of a `stealth_` account
#[repr(u32)]
#[derive(Debug, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub enum StealthKeys {
    V1(Box<StealthKeysV1>) = 1
}
impl StealthKeys {
    pub fn from_seed(seed: &SecretBytes<32>, i: u32, versions: StealthAccountVersions) -> Option<StealthKeys> {
        match versions.highest_supported_version() {
            Some(1) => Some(StealthKeys::V1(
                Box::new(StealthKeysV1::from_seed(seed, i, versions)) )),
            _ => None
        }
    }

    pub fn to_view_keys(&self) -> StealthViewKeys {
        self.into()
    }

    pub fn to_stealth_account(&self) -> StealthAccount {
        self.into()
    }

    /// Key of the account for "notification" transactions to be sent to, if applicable
    pub fn notification_key(&self) -> Key {
        unwrap_enum!(StealthKeys, self.notification_key())
    }

    /// Sign the `message` with the notification key, returning a `Signature`
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }
    /// Sign the `block` with the notification key, returning a `Signature`
    pub fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }

    /// Get the versions which this `stealth_` account supports
    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthKeys, self.get_versions())
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32> {
        unwrap_enum!(StealthKeys, self.receiver_ecdh(sender_account))
    }

    pub fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        unwrap_enum!(StealthKeys, self.derive_key_from_secret(secret, i))
    }

    pub fn derive_key(&self, sender_account: Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}



pub(crate) trait StealthViewKeysTrait: Sized + Zeroize + ZeroizeOnDrop {
    type AccountType: StealthAccountTrait;

    fn from_seed(view_seed: &SecretBytes<32>, master_spend: EdwardsPoint, i: u32, versions: StealthAccountVersions) -> Self;
    fn to_stealth_account(&self) -> Self::AccountType;

    fn notification_account(&self) -> Account;
    fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    fn get_versions(&self) -> StealthAccountVersions;

    fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32>;
    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account;
    fn derive_account(&self, sender_account: Account, i: u32) -> Account {
        self.derive_account_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}

/// The private view keys of a `stealth_` account
#[repr(u32)]
#[derive(Debug, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub enum StealthViewKeys {
    V1(Box<StealthViewKeysV1>) = 1
}
impl StealthViewKeys {
    pub fn from_keys(keys: StealthKeys) -> StealthViewKeys {
        keys.to_view_keys()
    }

    pub fn from_seed(seed: &SecretBytes<32>, master_spend: EdwardsPoint, i: u32, versions: StealthAccountVersions) -> Option<StealthViewKeys> {
        match versions.highest_supported_version() {
            Some(1) => Some(StealthViewKeys::V1(
                Box::new(StealthViewKeysV1::from_seed(seed, master_spend, i, versions)) )),
            _ => None
        }
    }

    pub fn to_stealth_account(&self) -> StealthAccount {
        self.into()
    }

    pub fn to_bytes(&self) -> SecretBytes<65> {
        self.into()
    }

    pub fn from_bytes(value: &SecretBytes<65>) -> Option<StealthViewKeys> {
        StealthViewKeys::try_from(value).ok()
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(StealthViewKeys, self.notification_account())
    }

    /// Check the validity of a signature made by the notification key
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    /// Get the versions which this `stealth_` account supports
    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthViewKeys, self.get_versions())
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32> {
        unwrap_enum!(StealthViewKeys, self.receiver_ecdh(sender_account))
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(StealthViewKeys, self.derive_account_from_secret(secret, i))
    }

    pub fn derive_account(&self, sender_account: Account, i: u32) -> Account {
        self.derive_account_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}

auto_from_impl!(From, StealthViewKeys, SecretBytes<65>);
auto_from_impl!(From, StealthKeys, StealthViewKeys);

impl From<&StealthViewKeys> for SecretBytes<65> {
    fn from(value: &StealthViewKeys) -> Self {
        unwrap_enum!(StealthViewKeys, value.into())
    }
}
impl TryFrom<SecretBytes<65>> for StealthViewKeys {
    type Error = ();

    fn try_from(value: SecretBytes<65>) -> Result<Self, ()> {
        (&value).try_into()
    }
}
impl TryFrom<&SecretBytes<65>> for StealthViewKeys {
    type Error = ();

    fn try_from(value: &SecretBytes<65>) -> Result<Self, ()> {
        let versions = StealthAccountVersions::decode_from_bits(value.as_ref()[0]);

        let value = match StealthViewKeysV1::try_from(value) {
            Ok(value) => value,
            Err(_) => return Err(())
        };
        match versions.highest_supported_version() {
            Some(1) => Ok(StealthViewKeys::V1(Box::new(value))),
            _ => Err(())
        }
    }
}
impl From<&StealthKeys> for StealthViewKeys {
    fn from(value: &StealthKeys) -> Self {
        match value {
            StealthKeys::V1(v1) => StealthViewKeys::V1( Box::new(v1.to_view_keys()) )
        }
    }
}



pub(crate) trait StealthAccountTrait: Sized + Zeroize + Display + PartialEq {
    type KeysType: StealthKeysTrait;

    fn from_keys(keys: Self::KeysType) -> Self;
    fn from_data(account: &str, data: &[u8]) -> Result<Self, NanoError>;
    fn from_string(account: &str) -> Result<Self, NanoError> {
        let data = base32::decode(&account[STEALTH_PREFIX_LEN..])
            .ok_or(NanoError::InvalidBase32)?;
        Self::from_data(account, &data)
    }

    fn notification_account(&self) -> Account;
    fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    fn get_versions(&self) -> StealthAccountVersions;
    fn is_valid(account: &str) -> bool {
        Self::from_string(account).is_ok()
    }

    fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32>;
    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account;
    fn derive_account(&self, sender_key: &Key, i: u32) -> Account {
        self.derive_account_from_secret(&self.sender_ecdh(sender_key), i)
    }
}

/// A `stealth_` account
#[repr(u32)]
#[derive(Debug, Zeroize, Clone, PartialEq, Eq)]
pub enum StealthAccount {
    V1(Box<StealthAccountV1>) = 1
}
impl StealthAccount {
    pub fn from_keys(keys: StealthKeys) -> StealthAccount {
        keys.to_stealth_account()
    }

    pub fn from_view_keys(keys: StealthViewKeys) -> StealthAccount {
        keys.to_stealth_account()
    }

    pub fn from_string(account: &str) -> Result<Self, NanoError> {
        // sanity check to prevent panic
        if account.len() < ADDRESS_CHARS_SAMPLE_END {
            return Err(NanoError::InvalidAddressLength)
        }
        if &account[..STEALTH_PREFIX_LEN] != STEALTH_ACCOUNT_PREFIX {
            return Err(NanoError::InvalidAddressPrefix)
        }
        let address_sample = &account[STEALTH_PREFIX_LEN..ADDRESS_CHARS_SAMPLE_END];
        let data = base32::decode(address_sample)
            .ok_or(NanoError::InvalidBase32)?;

        match version_bits!(data[0]).highest_supported_version() {
            Some(1) => Ok(StealthAccount::V1( Box::new(StealthAccountV1::from_string(account)?) )),
            _ => Err(NanoError::IncompatibleStealthVersions),
        }
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(StealthAccount, self.notification_account())
    }

    /// Check the validity of a signature made by the notification key
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    /// Get the versions which this `stealth_` account supports
    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthAccount, self.get_versions())
    }

    pub fn is_valid(account: &str) -> bool {
        Self::from_string(account).is_ok()
    }

    /// Calculate the shared secret between this account and the given key.
    pub fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32> {
        unwrap_enum!(StealthAccount, self.sender_ecdh(sender_key))
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(StealthAccount, self.derive_account_from_secret(secret, i))
    }

    pub fn derive_account(&self, sender_key: &Key, i: u32) -> Account {
        self.derive_account_from_secret(
            &self.sender_ecdh(sender_key), i
        )
    }
}

auto_from_impl!(From, StealthKeys, StealthAccount);
auto_from_impl!(From, StealthViewKeys, StealthAccount);

impl From<&StealthKeys> for StealthAccount {
    fn from(value: &StealthKeys) -> Self {
        match value {
            StealthKeys::V1(v1) => StealthAccount::V1( Box::new(v1.to_stealth_account()) )
        }
    }
}
impl From<&StealthViewKeys> for StealthAccount {
    fn from(value: &StealthViewKeys) -> Self {
        match value {
            StealthViewKeys::V1(v1) => StealthAccount::V1( Box::new(v1.to_stealth_account()) )
        }
    }
}
impl Display for StealthAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = unwrap_enum!(StealthAccount, self.to_string());
        write!(f, "{}", as_string)
    }
}



pub(super) trait AutoTestUtils: Sized {
    fn unwrap(self) -> Self {
        self
    }
}

macro_rules! stealth_address_tests {
    ($keys: ident, $view_keys: ident, $account: ident, $versions: expr, $addr: expr) => {
        impl AutoTestUtils for $keys {}
        impl AutoTestUtils for $account {}

        #[cfg(test)]
        mod tests {
            use crate::versions;
            use super::*;

            #[test]
            fn stealth_account() {
                let seed = SecretBytes::from([0; 32]);
                let key = $keys::from_seed(&seed, 0, $versions).unwrap();
                let view_keys = key.to_view_keys();
                let account = key.to_stealth_account();

                assert!(account.to_string() == $addr);
                assert!(account == $account::from_string($addr).unwrap());

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
                let recipient_account = recipient_keys.to_stealth_account();

                let recipient_derived = recipient_keys.derive_key(sender_account.clone(), 0).to_account();
                let recipient_vk_derived = recipient_view_keys.derive_account(sender_account, 0);
                let sender_derived = recipient_account.derive_account(&sender_keys, 0);

                assert!(recipient_derived == recipient_vk_derived);
                assert!(recipient_derived == sender_derived);
            }

            #[test]
            fn view_keys_bytes() {
                let seed = SecretBytes::from([42; 32]);
                let sender_view_keys_1 = $keys::from_seed(&seed, 99, $versions).unwrap().to_view_keys();

                let bytes: SecretBytes<65> = (&sender_view_keys_1).into();
                let sender_view_keys_2 = $view_keys::try_from(bytes).unwrap().into();

                assert!(sender_view_keys_1 == sender_view_keys_2);
            }
        }
    };
}
pub(crate) use stealth_address_tests;

#[cfg(test)]
use crate::constants::HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION;
stealth_address_tests!(
    StealthKeys, StealthViewKeys, StealthAccount,
    versions!(HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION),
    "stealth_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);