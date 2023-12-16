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

pub const STEALTH_PREFIX: &str = "stealth_";

pub fn get_genesis_account() -> Account {
    Account::try_from("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3").unwrap()
}