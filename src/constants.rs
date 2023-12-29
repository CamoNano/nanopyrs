use super::Account;

pub const ACCOUNT_PREFIX: &str = "nano_";

/// 0.000000000000000000000000000001 (10<sup>-30</sup>) Nano
pub const ONE_RAW:        u128 = 1;
/// 0.000000001 (10<sup>-9</sup>) Nano (lol)
pub const ONE_NANO_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000;
/// 0.000001 (10<sup>-6</sup>) Nano
pub const ONE_MICRO_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000;
/// 0.001 (10<sup>-3</sup>) Nano
pub const ONE_MILLI_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000_000;
/// 1 Nano
pub const ONE_NANO:       u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000_000_000;

pub fn get_genesis_account() -> Account {
    Account::try_from("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3").unwrap()
}

/// See [this](https://github.com/nanocurrency/nano-node/blob/220ac3de022c61ead2611a1fe2703b3fe4726eae/nano/secure/common.cpp#L103) for details
pub mod epoch_signers {
    use crate::Account;
    use super::*;

    /// This happens to be the genesis account
    pub fn get_v1_epoch_signer() -> Account {
        get_genesis_account()
    }

    pub fn get_v2_epoch_signer() -> Account {
        Account::try_from("nano_3qb6o6i1tkzr6jwr5s7eehfxwg9x6eemitdinbpi7u8bjjwsgqfj4wzser3x").unwrap()
    }
}

#[cfg(feature = "stealth")]
mod stealth {
    pub const STEALTH_ACCOUNT_PREFIX: &str = "stealth_";
    pub(crate) const ADDRESS_CHARS_SAMPLE_SIZE: usize = 8;

    pub(crate) const STEALTH_PREFIX_LEN: usize = STEALTH_ACCOUNT_PREFIX.len();
    pub(crate) const ADDRESS_CHARS_SAMPLE_END: usize = STEALTH_PREFIX_LEN + ADDRESS_CHARS_SAMPLE_SIZE;

    // The stealth account protocol has (currently) up to 8 versions.
    // Support for each version in an address is signaled by toggling bits in 1 byte, encoded in the address.
    // This improves backwards compatability, while still allowing new features to be added.
    pub const HIGHEST_KNOWN_STEALTH_PROTOCOL_VERSION: u8 = 0;
    pub(crate) const HIGHEST_POSSIBLE_STEALTH_PROTOCOL_VERSION: u8 = 7;

    /// intended to be used with `hashes::hazmat::get_category_seed`
    pub const SPEND_CONSTANTS_X_INDEX: u32 = 0;
    /// intended to be used with `hashes::hazmat::get_category_seed`
    pub const VIEW_CONSTANTS_X_INDEX: u32 = 1;
}
#[cfg(feature = "stealth")]
pub use stealth::*;