use super::Account;

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

// https://github.com/nanocurrency/nano-node/blob/220ac3de022c61ead2611a1fe2703b3fe4726eae/nano/secure/common.cpp#L103
pub mod epoch_signers {
    use crate::Account;
    use super::*;

    pub fn get_v1_epoch_signer() -> Account {
        get_genesis_account()
    }

    pub fn get_v2_epoch_signer() -> Account {
        Account::try_from("nano_3qb6o6i1tkzr6jwr5s7eehfxwg9x6eemitdinbpi7u8bjjwsgqfj4wzser3x").unwrap()
    }
}