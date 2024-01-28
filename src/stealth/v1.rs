use super::{
    stealth_address_tests, AutoTestUtils,
    StealthAccountTrait, StealthAccountVersions,
    StealthKeysTrait, StealthViewKeysTrait,
    get_standard_index
};
use crate::{
    auto_from_impl, base32,
    hashes::{
        blake2b512, blake2b_checksum, blake2b_scalar, get_stealth_spend_seed,
        get_stealth_view_seed,
        hazmat::{get_account_scalar, get_account_seed},
    },
    secret, try_compressed_from_slice, try_point_from_slice, version_bits, Account, Key, NanoError,
    Scalar, SecretBytes, Block
};
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::{CompressedEdwardsY, EdwardsPoint},
};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};

fn ecdh(key_1: &Scalar, key_2: &EdwardsPoint) -> SecretBytes<32> {
    secret!((key_1 * key_2).compress().to_bytes())
}

/// returns (spend, view)
fn get_partial_keys(view_seed: &SecretBytes<32>, i: u32) -> (Scalar, Scalar) {
    let account_seed = blake2b512(get_account_seed(view_seed, i).as_ref());
    (
        blake2b_scalar(&account_seed.as_ref()[..32]),
        blake2b_scalar(&account_seed.as_ref()[32..64]),
    )
}

fn points_to_account(
    versions: StealthAccountVersions,
    spend: EdwardsPoint,
    view: EdwardsPoint,
) -> StealthAccountV1 {
    let compressed_spend_key = spend.compress();
    let compressed_view_key = view.compress();

    let data = [
        [versions.encode_to_bits()].as_slice(),
        compressed_spend_key.as_bytes(),
        compressed_view_key.as_bytes(),
    ]
    .concat();
    let mut checksum = blake2b_checksum(&data);
    checksum.reverse();

    let mut account = "stealth_".to_string();
    let data = [data.as_slice(), &checksum].concat();
    account.push_str(&base32::encode(&data));

    StealthAccountV1 {
        account,
        versions,
        compressed_spend_key,
        compressed_view_key,
        point_spend_key: spend,
        point_view_key: view,
    }
}

fn account_from_data(account: &str, data: &[u8]) -> Result<StealthAccountV1, NanoError> {
    if account.len() != 120 {
        return Err(NanoError::InvalidAddressLength);
    }

    let versions = version_bits!(data[0]);
    let spend_key = &data[1..33];
    let view_key = &data[33..65];
    let checksum = &data[65..70];
    let mut calculated_checksum = blake2b_checksum(&data[..65]);
    calculated_checksum.reverse();

    if checksum != calculated_checksum {
        return Err(NanoError::InvalidAddressChecksum);
    }

    let compressed_spend_key = try_compressed_from_slice(spend_key)?;
    let compressed_view_key = try_compressed_from_slice(view_key)?;

    Ok(StealthAccountV1 {
        account: account.to_string(),
        versions,
        compressed_spend_key,
        compressed_view_key,
        point_spend_key: try_point_from_slice(spend_key)?,
        point_view_key: try_point_from_slice(view_key)?,
    })
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct StealthKeysV1 {
    versions: StealthAccountVersions,
    private_spend: Scalar,
    private_view: Scalar,
}
impl StealthKeysTrait for StealthKeysV1 {
    type ViewKeysType = StealthViewKeysV1;
    type AccountType = StealthAccountV1;

    fn from_seed(
        master_seed: &SecretBytes<32>,
        i: u32,
        versions: StealthAccountVersions,
    ) -> StealthKeysV1 {
        let master_spend = get_account_scalar(&get_stealth_spend_seed(master_seed), 0);
        let (partial_spend, private_view) =
            get_partial_keys(&get_stealth_view_seed(master_seed), i);
        StealthKeysV1 {
            versions,
            private_spend: master_spend + partial_spend,
            private_view,
        }
    }

    fn to_view_keys(&self) -> Self::ViewKeysType {
        let spend = &self.private_spend * G;
        StealthViewKeysV1 {
            versions: self.versions,
            compressed_spend_key: spend.compress(),
            point_spend_key: spend,
            private_view: self.private_view.clone(),
        }
    }

    fn to_stealth_account(&self) -> Self::AccountType {
        points_to_account(
            self.versions,
            &self.private_spend * G,
            &self.private_view * G,
        )
    }

    fn notification_key(&self) -> Key {
        Key::from_scalar(self.private_spend.clone())
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn receiver_ecdh(&self, sender_account: &Account) -> SecretBytes<32> {
        ecdh(&self.private_view, &sender_account.point)
    }

    fn derive_key_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        Key::from(&self.private_spend + get_account_scalar(secret, i))
    }

    fn derive_key_from_block(&self, block: &Block) -> Key {
        self.derive_key(&block.account, get_standard_index(block))
    }
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct StealthViewKeysV1 {
    versions: StealthAccountVersions,
    compressed_spend_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    private_view: Scalar,
}
impl StealthViewKeysTrait for StealthViewKeysV1 {
    type AccountType = StealthAccountV1;

    fn from_seed(
        view_seed: &SecretBytes<32>,
        master_spend: EdwardsPoint,
        i: u32,
        versions: StealthAccountVersions,
    ) -> StealthViewKeysV1 {
        let (private_spend, private_view) = get_partial_keys(view_seed, i);
        let point_spend_key = master_spend + (private_spend * G);
        StealthViewKeysV1 {
            versions,
            compressed_spend_key: point_spend_key.compress(),
            point_spend_key,
            private_view,
        }
    }

    fn to_stealth_account(&self) -> StealthAccountV1 {
        points_to_account(self.versions, self.point_spend_key, &self.private_view * G)
    }

    fn notification_account(&self) -> Account {
        Account::from_both_points(&self.point_spend_key, &self.compressed_spend_key)
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn receiver_ecdh(&self, sender_key: &Account) -> SecretBytes<32> {
        ecdh(&self.private_view, &sender_key.point)
    }

    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }

    fn derive_account_from_block(&self, block: &Block) -> Account {
        self.derive_account(&block.account, get_standard_index(block))
    }
}

auto_from_impl!(From: StealthViewKeysV1 => SecretBytes<65>);
auto_from_impl!(TryFrom: SecretBytes<65> => StealthViewKeysV1);

impl From<&StealthViewKeysV1> for SecretBytes<65> {
    fn from(value: &StealthViewKeysV1) -> Self {
        let bytes: [u8; 65] = [
            [value.versions.encode_to_bits()].as_slice(),
            value.compressed_spend_key.as_bytes(),
            value.private_view.as_bytes(),
        ]
        .concat()
        .try_into()
        .unwrap();
        SecretBytes::from(bytes)
    }
}
impl TryFrom<&SecretBytes<65>> for StealthViewKeysV1 {
    type Error = NanoError;

    fn try_from(value: &SecretBytes<65>) -> Result<Self, NanoError> {
        let bytes = value.as_ref();

        let versions = StealthAccountVersions::decode_from_bits(bytes[0]);
        let compressed_spend_key = try_compressed_from_slice(&bytes[1..33])?;
        let point_spend_key = try_point_from_slice(&bytes[1..33])?;
        let private_view = Scalar::from_canonical_bytes(bytes[33..].as_ref().try_into().unwrap())?;

        Ok(StealthViewKeysV1 {
            versions,
            compressed_spend_key,
            point_spend_key,
            private_view,
        })
    }
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct StealthAccountV1 {
    account: String,
    versions: StealthAccountVersions,
    compressed_spend_key: CompressedEdwardsY,
    compressed_view_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    point_view_key: EdwardsPoint,
}
impl StealthAccountTrait for StealthAccountV1 {
    type KeysType = StealthKeysV1;

    fn from_keys(keys: Self::KeysType) -> StealthAccountV1 {
        keys.to_stealth_account()
    }

    fn from_data(account: &str, data: &[u8]) -> Result<StealthAccountV1, NanoError> {
        account_from_data(account, data)
    }

    fn notification_account(&self) -> Account {
        Account::from_both_points(&self.point_spend_key, &self.compressed_spend_key)
    }

    fn get_versions(&self) -> StealthAccountVersions {
        self.versions
    }

    fn sender_ecdh(&self, sender_key: &Key) -> SecretBytes<32> {
        ecdh(sender_key.as_scalar(), &self.point_view_key)
    }

    fn derive_account_from_secret(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }

    fn derive_account_from_block(&self, block: &Block, key: &Key) -> Account {
        self.derive_account(key, get_standard_index(block))
    }
}
impl Display for StealthAccountV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.account)
    }
}

stealth_address_tests!(
    StealthKeysV1, StealthViewKeysV1, StealthAccountV1,
    versions!(1),
    "stealth_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);
