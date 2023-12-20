mod version;
mod v0;

use crate::version_bits;
use crate::{
    base32,
    NanoError, Key, Account, Signature, SecretBytes,
    constants::{STEALTH_PREFIX, STEALTH_PREFIX_LEN, ADDRESS_CHARS_SAMPLE_END}
};
use v0::{StealthKeysV0, StealthViewKeysV0, StealthAccountV0};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};
use curve25519_dalek::edwards::EdwardsPoint;

pub use version::StealthAccountVersions;

macro_rules! unwrap_enum {
    (StealthKeys, $instance: ident) => {
        match $instance {
            StealthKeys::V0(v1) => v1
        }
    };
    (StealthViewKeys, $instance: ident) => {
        match $instance {
            StealthViewKeys::V0(v1) => v1
        }
    };
    (StealthAccount, $instance: ident) => {
        match $instance {
            StealthAccount::V0(v1) => v1
        }
    };
}



pub(crate) trait StealthKeysTrait: Sized + Zeroize + ZeroizeOnDrop  {
    type ViewKeysType: StealthViewKeysTrait;
    type AccountType: StealthAccountTrait;

    fn from_seed(seed: &SecretBytes<32>, i: u32, versions: StealthAccountVersions) -> Self;
    fn to_view_keys(&self) -> Self::ViewKeysType;
    fn to_stealth_account(&self) -> Self::AccountType;

    /// Account for "notification" transactions to be sent to, if applicable
    fn notification_key(&self) -> Key;
    fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }

    fn get_versions(&self) -> StealthAccountVersions;

    fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32>;
    fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key;
    fn derive_key(&self, sender_account: Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub enum StealthKeys {
    V0(StealthKeysV0)
}
impl StealthKeys {
    pub fn from_seed(seed: &SecretBytes<32>, i: u32, versions: StealthAccountVersions) -> Result<StealthKeys, NanoError> {
        match versions.highest_supported_version() {
            Some(0) => Ok(StealthKeys::V0(StealthKeysV0::from_seed(seed, i, versions))),
            _ => Err(NanoError::InvalidVersions(versions))
        }
    }

    pub fn to_view_keys(&self) -> StealthViewKeys {
        self.into()
    }

    pub fn to_stealth_account(&self) -> StealthAccount {
        self.into()
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_key(&self) -> Key {
        unwrap_enum!(StealthKeys, self).notification_key()
    }

    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.notification_key().sign_message(message)
    }

    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthKeys, self).get_versions()
    }

    pub fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32> {
        unwrap_enum!(StealthKeys, self).receiver_ecdh(sender_account)
    }

    pub fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        unwrap_enum!(StealthKeys, self).derive_key_from_secret(secret, i)
    }

    pub fn derive_key(&self, sender_account: Account, i: u32) -> Key {
        self.derive_key_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}



pub(crate) trait StealthViewKeysTrait: Sized + Zeroize + ZeroizeOnDrop  {
    type AccountType: StealthAccountTrait;

    fn from_seed(view_seed: &SecretBytes<32>, master_spend: EdwardsPoint, i: u32, versions: StealthAccountVersions) -> Self;
    fn to_stealth_account(&self) -> Self::AccountType;

    /// Account for "notification" transactions to be sent to, if applicable
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

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub enum StealthViewKeys {
    V0(StealthViewKeysV0)
}
impl StealthViewKeys {
    pub fn from_seed(seed: &SecretBytes<32>, master_spend: EdwardsPoint, i: u32, versions: StealthAccountVersions) -> Result<StealthViewKeys, NanoError> {
        match versions.highest_supported_version() {
            Some(0) => Ok(StealthViewKeys::V0(StealthViewKeysV0::from_seed(seed, master_spend, i, versions))),
            _ => Err(NanoError::InvalidVersions(versions))
        }
    }

    pub fn to_stealth_account(&self) -> StealthAccount {
        self.into()
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(StealthViewKeys, self).notification_account()
    }

    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthViewKeys, self).get_versions()
    }

    pub fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32> {
        unwrap_enum!(StealthViewKeys, self).receiver_ecdh(sender_account)
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(StealthViewKeys, self).derive_account_from_secret(secret, i)
    }

    pub fn derive_account(&self, sender_account: Account, i: u32) -> Account {
        self.derive_account_from_secret(&self.receiver_ecdh(sender_account), i)
    }
}
impl From<StealthKeys> for StealthViewKeys {
    fn from(value: StealthKeys) -> Self {
        (&value).into()
    }
}
impl From<&StealthKeys> for StealthViewKeys {
    fn from(value: &StealthKeys) -> Self {
        match value {
            StealthKeys::V0(v1) => StealthViewKeys::V0(v1.to_view_keys())
        }
    }
}



pub(crate) trait StealthAccountTrait: Sized + Zeroize + Display + PartialEq {
    type KeysType: StealthKeysTrait;

    fn from_keys(keys: Self::KeysType) -> Self;
    fn from_data(account: &str, data: &[u8]) -> Result<Self, NanoError>;
    fn from_string(account: &str) -> Result<Self, NanoError> {
        let data = base32::decode(&account[8..])
            .ok_or(NanoError::InvalidBase32)?;
        Self::from_data(account, &data)
    }

    /// Account for "notification" transactions to be sent to, if applicable
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

#[derive(Debug, Zeroize, Clone, PartialEq)]
pub enum StealthAccount {
    V0(StealthAccountV0)
}
impl StealthAccount {
    pub fn from_keys(keys: StealthKeys) -> StealthAccount {
        keys.to_stealth_account()
    }

    pub fn from_string(account: &str) -> Result<Self, NanoError> {
        // sanity check to prevent panic
        if account.len() < ADDRESS_CHARS_SAMPLE_END {
            return Err(NanoError::InvalidLength)
        }
        if &account[..STEALTH_PREFIX_LEN] != STEALTH_PREFIX {
            return Err(NanoError::InvalidFormatting)
        }
        let address_sample = &account[STEALTH_PREFIX_LEN..ADDRESS_CHARS_SAMPLE_END];
        let data = base32::decode(address_sample)
            .ok_or(NanoError::InvalidBase32)?;

        let versions = version_bits!(data[0]);
        match versions.highest_supported_version() {
            Some(0) => Ok(StealthAccount::V0(StealthAccountV0::from_string(account)?)),
            _ => Err(NanoError::InvalidVersions(versions)),
        }
    }

    /// Account for "notification" transactions to be sent to, if applicable
    pub fn notification_account(&self) -> Account {
        unwrap_enum!(StealthAccount, self).notification_account()
    }

    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.notification_account().is_valid_signature(message, signature)
    }

    pub fn get_versions(&self) -> StealthAccountVersions {
        unwrap_enum!(StealthAccount, self).get_versions()
    }

    pub fn is_valid(account: &str) -> bool {
        Self::from_string(account).is_ok()
    }

    pub fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32> {
        unwrap_enum!(StealthAccount, self).sender_ecdh(sender_key)
    }

    pub fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        unwrap_enum!(StealthAccount, self).derive_account_from_secret(secret, i)
    }

    pub fn derive_account(&self, sender_key: &Key, i: u32) -> Account {
        self.derive_account_from_secret(
            &self.sender_ecdh(sender_key), i
        )
    }
}
impl From<StealthKeys> for StealthAccount {
    fn from(value: StealthKeys) -> Self {
        (&value).into()
    }
}
impl From<&StealthKeys> for StealthAccount {
    fn from(value: &StealthKeys) -> Self {
        match value {
            StealthKeys::V0(v1) => StealthAccount::V0(v1.to_stealth_account())
        }
    }
}
impl From<StealthViewKeys> for StealthAccount {
    fn from(value: StealthViewKeys) -> Self {
        (&value).into()
    }
}
impl From<&StealthViewKeys> for StealthAccount {
    fn from(value: &StealthViewKeys) -> Self {
        match value {
            StealthViewKeys::V0(v1) => StealthAccount::V0(v1.to_stealth_account())
        }
    }
}
impl Display for StealthAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = unwrap_enum!(StealthAccount, self).to_string();
        write!(f, "{}", as_string)
    }
}



pub(super) trait AutoTestUtils: Sized {
    fn unwrap(self) -> Self {
        self
    }
}

macro_rules! stealth_address_tests {
    ($keys: ident, $versions: expr, $addr: expr) => {
        impl AutoTestUtils for $keys {}

        #[cfg(test)]
        mod tests {
            use crate::versions;
            use super::*;

            #[test]
            fn stealth_account() {
                let seed = SecretBytes::from(&mut [0; 32]);
                let key = $keys::from_seed(&seed, 0, $versions).unwrap();
                let view_keys = key.to_view_keys();
                let account = key.to_stealth_account();

                assert!(account.to_string() == $addr);

                assert!($versions == key.get_versions());
                assert!($versions == view_keys.get_versions());
                assert!($versions == account.get_versions());
            }

            #[test]
            fn notification_account() {
                let seed = SecretBytes::from(&mut [0; 32]);
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
                let seed = SecretBytes::from(&mut [127; 32]);

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
        }
    };
}
pub(crate) use stealth_address_tests;

#[cfg(test)]
use crate::constants::HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION;
stealth_address_tests!(
    StealthKeys,
    versions!(HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION),
    "stealth_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);