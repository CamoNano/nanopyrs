use super::Account;

pub const ACCOUNT_PREFIX: &str = "nano_";

/// 0.000000000000000000000000000001 (10<sup>-30</sup>) Nano
pub const ONE_RAW: u128 = 1;
/// 0.000000001 (10<sup>-9</sup>) Nano (lol)
pub const ONE_NANO_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000;
/// 0.000001 (10<sup>-6</sup>) Nano
pub const ONE_MICRO_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000;
/// 0.001 (10<sup>-3</sup>) Nano
pub const ONE_MILLI_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000_000;
/// 1 Nano
pub const ONE_NANO: u128 = ONE_RAW * 1_000_000_000_000_000_000_000_000_000_000;

pub fn get_genesis_account() -> Account {
    Account::try_from("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3").unwrap()
}

/// See [here](https://github.com/nanocurrency/nano-node/blob/220ac3de022c61ead2611a1fe2703b3fe4726eae/nano/secure/common.cpp#L103) for details
pub mod epoch_signers {
    use super::*;
    use crate::Account;

    /// This happens to be the genesis account
    pub fn get_v1_epoch_signer() -> Account {
        get_genesis_account()
    }

    pub fn get_v2_epoch_signer() -> Account {
        Account::try_from("nano_3qb6o6i1tkzr6jwr5s7eehfxwg9x6eemitdinbpi7u8bjjwsgqfj4wzser3x")
            .unwrap()
    }
}

#[cfg(feature = "camo")]
mod camo {
    use super::ONE_MICRO_NANO;
    use crate::camo::CamoVersion;

    pub(crate) const CAMO_ACCOUNT_PREFIX: &str = "camo_";
    pub(crate) const ADDRESS_CHARS_SAMPLE_SIZE: usize = 8;

    pub(crate) const CAMO_PREFIX_LEN: usize = CAMO_ACCOUNT_PREFIX.len();
    pub(crate) const ADDRESS_CHARS_SAMPLE_END: usize = CAMO_PREFIX_LEN + ADDRESS_CHARS_SAMPLE_SIZE;

    /// The highest supported protocol version for `camo_` accounts.
    ///
    /// Currently, only version `1` is supported.
    pub const HIGHEST_KNOWN_CAMO_PROTOCOL_VERSION: u8 = 1;

    pub(crate) const ALL_POSSIBLE_CAMO_VERSIONS: &[CamoVersion] = &[
        CamoVersion::One,
        CamoVersion::Two,
        CamoVersion::Three,
        CamoVersion::Four,
        CamoVersion::Five,
        CamoVersion::Six,
        CamoVersion::Seven,
        CamoVersion::Eight,
    ];

    pub(crate) const ALL_SUPPORTED_CAMO_VERSIONS: &[CamoVersion] = &[CamoVersion::One];

    /// 0.0005 (5 * 10<sup>-4</sup>) Nano.
    ///
    /// The minimum amount of coins that should be *sent* in a Camo transaction.
    pub const CAMO_SENDER_DUST_THRESHOLD: u128 = ONE_MICRO_NANO * 500;
    /// 0.00049 (4.9 * 10<sup>-4</sup>) Nano.
    ///
    /// The minimum amount of coins that should be *received* in a Camo transaction.
    pub const CAMO_RECIPIENT_DUST_THRESHOLD: u128 = ONE_MICRO_NANO * 490;

    /// intended to be used with `hashes::hazmat::get_category_seed`
    pub const SPEND_CONSTANTS_X_INDEX: u32 = 0;
    /// intended to be used with `hashes::hazmat::get_category_seed`
    pub const VIEW_CONSTANTS_X_INDEX: u32 = 1;
}
#[cfg(feature = "camo")]
pub use camo::*;

#[cfg(test)]
#[cfg(feature = "serde")]
pub(crate) const USIZE_LEN: usize = std::mem::size_of::<usize>();
