mod addressv1;
mod notification;
mod version;

use crate::{
    auto_from_impl, base32,
    constants::{ADDRESS_CHARS_SAMPLE_END, CAMO_ACCOUNT_PREFIX, CAMO_PREFIX_LEN},
    version_bits, Account, Block, Key, NanoError, SecretBytes, Signature,
};
use addressv1::{CamoAccountType1, CamoKeysType1, CamoViewKeysType1};
use curve25519_dalek::edwards::EdwardsPoint;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use notification::{Notification, NotificationV1};
pub use version::{CamoVersion, CamoVersions};

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

/// The private keys of a `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoKeys {
    V1(Box<CamoKeysType1>) = 1,
}
impl CamoKeys {
    /// Returns `None` if no supported version is given
    pub fn from_seed(seed: &SecretBytes<32>, i: u32, versions: CamoVersions) -> Option<CamoKeys> {
        match versions.highest_supported_version() {
            Some(CamoVersion::One | CamoVersion::Two) => Some(CamoKeys::V1(Box::new(
                CamoKeysType1::from_seed(seed, i, versions),
            ))),
            _ => None,
        }
    }

    /// Get the camo protocol versions that this address supports
    pub fn camo_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoKeys, self.camo_versions())
    }

    pub fn to_view_keys(&self) -> CamoViewKeys {
        self.into()
    }

    pub fn to_camo_account(&self) -> CamoAccount {
        self.into()
    }

    /// The private spend key of this camo address.
    ///
    /// Also the key of the account for "notification" transactions to be sent to, if applicable.
    pub fn signer_key(&self) -> Key {
        unwrap_enum!(CamoKeys, self.signer_key())
    }

    /// Sign the `message` with the spend key, returning a `Signature`
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.signer_key().sign_message(message)
    }
    /// Sign the `block` with the spend key, returning a `Signature`
    pub fn sign_block(&self, block: &Block) -> Signature {
        self.sign_message(&block.hash())
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, notification: &Notification) -> SecretBytes<32> {
        unwrap_enum!(CamoKeys, self.receiver_ecdh(notification))
    }

    /// Use `receiver_ecdh()` to obtain the `secret`
    pub fn derive_key(&self, secret: &SecretBytes<32>) -> Key {
        unwrap_enum!(CamoKeys, self.derive_key(secret))
    }
}

/// The private view keys of a `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoViewKeys {
    V1(Box<CamoViewKeysType1>) = 1,
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
            Some(CamoVersion::One | CamoVersion::Two) => Some(CamoViewKeys::V1(Box::new(
                CamoViewKeysType1::from_seed(seed, master_spend, i, versions),
            ))),
            _ => None,
        }
    }

    /// Get the camo protocol versions that this address supports
    pub fn camo_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoViewKeys, self.camo_versions())
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

    /// The public spend key of this camo address.
    ///
    /// Also the account for "notification" transactions to be sent to, if applicable.
    pub fn signer_account(&self) -> Account {
        unwrap_enum!(CamoViewKeys, self.signer_account())
    }

    /// Check the validity of a signature made by the notification key
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.signer_account()
            .is_valid_signature(message, &signature)
    }

    /// Calculate the shared secret between this key and the given account.
    pub fn receiver_ecdh(&self, notification: &Notification) -> SecretBytes<32> {
        unwrap_enum!(CamoViewKeys, self.receiver_ecdh(notification))
    }

    /// Use `receiver_ecdh()` to obtain the `secret`
    pub fn derive_account(&self, secret: &SecretBytes<32>) -> Account {
        unwrap_enum!(CamoViewKeys, self.derive_account(secret))
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

        let value = match CamoViewKeysType1::try_from(value) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        match versions.highest_supported_version() {
            Some(CamoVersion::One | CamoVersion::Two) => Ok(CamoViewKeys::V1(Box::new(value))),
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

/// A `camo_` account
#[repr(u32)]
#[derive(Debug, Clone, Hash, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CamoAccount {
    V1(Box<CamoAccountType1>) = 1,
}
impl CamoAccount {
    pub fn from_keys(keys: CamoKeys) -> CamoAccount {
        keys.to_camo_account()
    }

    pub fn from_view_keys(keys: CamoViewKeys) -> CamoAccount {
        keys.to_camo_account()
    }

    /// Get the camo protocol versions that this address supports
    pub fn camo_versions(&self) -> CamoVersions {
        unwrap_enum!(CamoAccount, self.camo_versions())
    }

    /// The public spend key of this camo address.
    ///
    /// Also the account for "notification" transactions to be sent to, if applicable.
    pub fn signer_account(&self) -> Account {
        unwrap_enum!(CamoAccount, self.signer_account())
    }

    /// Check the validity of a signature made by the spend key of this camo address
    pub fn is_valid_signature(&self, message: &[u8], signature: Signature) -> bool {
        self.signer_account()
            .is_valid_signature(message, &signature)
    }

    pub fn is_valid(account: &str) -> bool {
        Self::from_str(account).is_ok()
    }

    /// Calculate the shared secret between this account and the given key.
    ///
    /// `sender_frontier` is used to ensure that all generated keys are unique per-camo-payment.
    pub fn sender_ecdh(
        &self,
        sender_key: &Key,
        sender_frontier: [u8; 32],
    ) -> (SecretBytes<32>, Notification) {
        unwrap_enum!(CamoAccount, self.sender_ecdh(sender_key, sender_frontier))
    }

    /// Use `sender_ecdh()` to obtain the `secret`
    pub fn derive_account(&self, secret: &SecretBytes<32>) -> Account {
        unwrap_enum!(CamoAccount, self.derive_account(secret))
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
            Some(CamoVersion::One | CamoVersion::Two) => {
                Ok(CamoAccount::V1(Box::new(CamoAccountType1::from_str(s)?)))
            }
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

#[cfg(test)]
mod protocol_docs_tests {
    use super::*;

    #[test]
    fn example_camo_address() {
        let expected = "camo_168be68tsxk1o8xferck89gj75kzk8fpbhote77ed1db975htuf11psgpwq9wabcxdjssycim6tidgkau48x6tgcqnsnxj341mamjpoy8umaz45c".parse().unwrap();
        let seed = SecretBytes::from([200; 32]);
        let keys = CamoKeys::from_seed(&seed, 5, version_bits!(0x01)).unwrap();
        assert!(keys.to_camo_account() == expected);
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

                assert!($versions == key.camo_versions());
                assert!($versions == view_keys.camo_versions());
                assert!($versions == account.camo_versions());
            }

            #[test]
            fn signer_account() {
                let seed = SecretBytes::from([0; 32]);
                let keys = $keys::from_seed(&seed, 0, $versions).unwrap();
                let view_keys = keys.to_view_keys();
                let account = keys.to_view_keys();

                let keys_account = keys.signer_key().to_account();
                let view_keys_account = view_keys.signer_account();
                let signer_account = account.signer_account();

                assert!(keys_account == view_keys_account);
                assert!(keys_account == signer_account);
            }

            #[test]
            fn derive_account() {
                let seed = SecretBytes::from([127; 32]);

                let sender_keys = Key::from_seed(&seed, 0);

                let recipient_keys = $keys::from_seed(&seed, 99, $versions).unwrap();
                let recipient_view_keys = recipient_keys.to_view_keys();
                let recipient_account = recipient_keys.to_camo_account();

                let (sender_ecdh, notification) =
                    recipient_account.sender_ecdh(&sender_keys, [50; 32]);
                let sender_derived = recipient_account.derive_account(&sender_ecdh);

                let recipient_ecdh = recipient_keys.receiver_ecdh(&notification);
                let recipient_derived = recipient_keys.derive_key(&recipient_ecdh).to_account();
                let recipient_vk_derived = recipient_view_keys.derive_account(&recipient_ecdh);

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
