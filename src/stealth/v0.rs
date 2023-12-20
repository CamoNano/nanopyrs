use crate::{
    secret, version_bits,
    base32,
    NanoError, SecretBytes, Scalar, Key, Account,
    try_compressed_from_slice, try_point_from_slice,
    hashes::{
        blake2b512, blake2b_checksum, blake2b_scalar, get_spend_seed, get_view_seed,
        hazmat::{get_account_seed, get_account_scalar}
    }
};
use super::{
    StealthKeysTrait, StealthViewKeysTrait, StealthAccountTrait, StealthAccountVersions,
    AutoTestUtils, stealth_address_tests
};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};
use curve25519_dalek::{
    edwards::{EdwardsPoint, CompressedEdwardsY},
    constants::ED25519_BASEPOINT_POINT as G
};

fn ecdh(key_1: &Scalar, key_2: EdwardsPoint) -> SecretBytes<32> {
    secret!(&mut (key_1 * key_2).compress().to_bytes())
}

/// returns (spend, view)
fn get_private_keys(view_seed: &SecretBytes<32>, i: u32) -> (Scalar, Scalar) {
    let view_seed = blake2b512(get_account_seed(view_seed, i).as_ref());
    (
        blake2b_scalar(&view_seed.as_ref()[..32]),
        blake2b_scalar(&view_seed.as_ref()[32..64])
    )
}

fn points_to_account(versions: StealthAccountVersions, spend: EdwardsPoint, view: EdwardsPoint) -> StealthAccountV0 {
    let compressed_spend_key = spend.compress();
    let compressed_view_key = view.compress();

    let data = [
        [versions.encode_to_bits()].as_slice(),
        compressed_spend_key.as_bytes(),
        compressed_view_key.as_bytes()
    ].concat();
    let mut checksum = blake2b_checksum(&data);
    checksum.reverse();

    let mut account = "stealth_".to_string();
    let data = [data.as_slice(), &checksum].concat();
    account.push_str(&base32::encode(&data));

    StealthAccountV0 {
        account,
        versions,
        compressed_spend_key,
        compressed_view_key,
        point_spend_key: spend,
        point_view_key: view
    }
}

fn account_from_data(account: &str, data: &[u8]) -> Result<StealthAccountV0, NanoError> {
    if account.len() != 120 {
        return Err(NanoError::InvalidLength)
    }

    let versions = version_bits!(data[0]);
    let spend_key = &data[1..33];
    let view_key = &data[33..65];
    let checksum = &data[65..70];
    let mut calculated_checksum = blake2b_checksum(&data[..65]);
    calculated_checksum.reverse();

    if checksum != calculated_checksum {
        return Err(NanoError::InvalidChecksum)
    }

    let compressed_spend_key = try_compressed_from_slice(spend_key)?;
    let compressed_view_key = try_compressed_from_slice(view_key)?;

    Ok(
        StealthAccountV0 {
            account: account.to_string(),
            versions,
            compressed_spend_key,
            compressed_view_key,
            point_spend_key: try_point_from_slice(spend_key)?,
            point_view_key: try_point_from_slice(view_key)?
        }
    )
}



#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct StealthKeysV0 {
    versions: StealthAccountVersions,
    private_spend: Scalar,
    private_view: Scalar,
}
impl StealthKeysTrait for StealthKeysV0 {
    type ViewKeysType = StealthViewKeysV0;
    type AccountType = StealthAccountV0;

    fn from_seed(master_seed: &SecretBytes<32>, i: u32, versions: StealthAccountVersions) -> StealthKeysV0 {
        let master_spend = get_account_scalar(&get_spend_seed(master_seed), 0);
        let (private_spend, private_view) = get_private_keys(&get_view_seed(master_seed), i);
        StealthKeysV0 {
            versions,
            private_spend: master_spend + private_spend,
            private_view
        }
    }

    fn to_view_keys(&self) -> Self::ViewKeysType {
        let spend = &self.private_spend * G;
        StealthViewKeysV0 {
            versions: self.versions,
            compressed_spend_key: spend.compress(),
            point_spend_key: spend,
            private_view: self.private_view.dangerous_clone()
        }
    }

    fn to_stealth_account(&self) -> Self::AccountType {
        points_to_account(self.versions, &self.private_spend * G, &self.private_view * G)
    }

    fn notification_key(&self) -> Key {
        Key::from_scalar(self.private_spend.dangerous_clone())
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn receiver_ecdh(&self, sender_account: Account) -> SecretBytes<32> {
        ecdh(&self.private_view, sender_account.point)
    }

    fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        Key::from(&self.private_spend + get_account_scalar(secret, i))
    }
}



#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct StealthViewKeysV0 {
    versions: StealthAccountVersions,
    compressed_spend_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    private_view: Scalar,
}
impl StealthViewKeysTrait for StealthViewKeysV0 {
    type AccountType = StealthAccountV0;

    fn from_seed(view_seed: &SecretBytes<32>, master_spend: EdwardsPoint, i: u32, versions: StealthAccountVersions) -> StealthViewKeysV0 {
        let (private_spend, private_view) = get_private_keys(view_seed, i);
        let point_spend_key = master_spend + (private_spend * G);
        StealthViewKeysV0 {
            versions,
            compressed_spend_key: point_spend_key.compress(),
            point_spend_key,
            private_view
        }
    }

    fn to_stealth_account(&self) -> StealthAccountV0 {
        points_to_account(self.versions, self.point_spend_key, &self.private_view * G)
    }

    fn notification_account(&self) -> Account {
        Account::from_compressed(&self.compressed_spend_key).unwrap()
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn receiver_ecdh(&self, sender_key: Account) -> SecretBytes<32> {
        ecdh(&self.private_view, sender_key.point)
    }

    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }
}



#[derive(Debug, Clone, Zeroize, PartialEq)]
pub struct StealthAccountV0 {
    account: String,
    versions: StealthAccountVersions,
    compressed_spend_key: CompressedEdwardsY,
    compressed_view_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    point_view_key: EdwardsPoint
}
impl StealthAccountTrait for StealthAccountV0 {
    type KeysType = StealthKeysV0;

    fn from_keys(keys: Self::KeysType) -> StealthAccountV0 {
        keys.to_stealth_account()
    }

    fn from_data(account: &str, data: &[u8]) -> Result<StealthAccountV0, NanoError> {
        account_from_data(account, data)
    }

    fn notification_account(&self) -> Account {
        Account::from_compressed(&self.compressed_spend_key).unwrap()
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32> {
        ecdh(sender_key.as_scalar(), self.point_view_key)
    }

    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }
}
impl Display for StealthAccountV0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",  self.account.clone())
    }
}



stealth_address_tests!(
    StealthKeysV0, StealthAccountV0,
    versions!(0),
    "stealth_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);