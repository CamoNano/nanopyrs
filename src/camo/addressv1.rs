use super::{
    camo_address_tests, AutoTestUtils, CamoVersion, CamoVersions, Notification, CAMO_PREFIX_LEN,
};
use crate::{
    auto_from_impl, base32,
    hashes::{
        blake2b512, blake2b_checksum, blake2b_scalar, get_camo_spend_seed, get_camo_view_seed,
        hazmat::{get_account_scalar, get_account_seed},
    },
    secret, try_compressed_from_slice, try_point_from_slice, version_bits, Account, Key, NanoError,
    Scalar, SecretBytes,
};
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::{CompressedEdwardsY, EdwardsPoint},
};
use std::fmt::Display;
use std::hash::Hash;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const ADDRESS_LENGTH: usize = 117;

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
    versions: CamoVersions,
    spend: EdwardsPoint,
    view: EdwardsPoint,
) -> CamoAccountType1 {
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

    let mut account = "camo_".to_string();
    let data = [data.as_slice(), &checksum].concat();
    account.push_str(&base32::encode(&data));

    CamoAccountType1 {
        account,
        versions,
        compressed_spend_key,
        compressed_view_key,
        point_spend_key: spend,
        point_view_key: view,
    }
}

fn account_from_data(account: &str, data: &[u8]) -> Result<CamoAccountType1, NanoError> {
    if account.len() != ADDRESS_LENGTH {
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

    Ok(CamoAccountType1 {
        account: account.to_string(),
        versions,
        compressed_spend_key,
        compressed_view_key,
        point_spend_key: try_point_from_slice(spend_key)?,
        point_view_key: try_point_from_slice(view_key)?,
    })
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CamoKeysType1 {
    versions: CamoVersions,
    private_spend: Scalar,
    private_view: Scalar,
}
impl CamoKeysType1 {
    pub fn from_seed(
        master_seed: &SecretBytes<32>,
        i: u32,
        versions: CamoVersions,
    ) -> CamoKeysType1 {
        let master_spend = get_account_scalar(&get_camo_spend_seed(master_seed), 0);
        let (partial_spend, private_view) = get_partial_keys(&get_camo_view_seed(master_seed), i);
        CamoKeysType1 {
            versions,
            private_spend: master_spend + partial_spend,
            private_view,
        }
    }

    pub fn camo_versions(&self) -> CamoVersions {
        self.versions
    }

    pub fn to_view_keys(&self) -> CamoViewKeysType1 {
        let spend = &self.private_spend * G;
        CamoViewKeysType1 {
            versions: self.versions,
            compressed_spend_key: spend.compress(),
            point_spend_key: spend,
            private_view: self.private_view.clone(),
        }
    }

    pub fn to_camo_account(&self) -> CamoAccountType1 {
        points_to_account(
            self.versions,
            &self.private_spend * G,
            &self.private_view * G,
        )
    }

    pub fn signer_key(&self) -> Key {
        Key::from_scalar(self.private_spend.clone())
    }

    pub fn receiver_ecdh(&self, notification: &Notification) -> SecretBytes<32> {
        let point = match notification {
            Notification::V1(v1) => v1.representative_payload.point,
        };
        ecdh(&self.private_view, &point)
    }

    pub fn derive_key(&self, secret: &SecretBytes<32>, i: u32) -> Key {
        Key::from(&self.private_spend + get_account_scalar(secret, i))
    }
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct CamoViewKeysType1 {
    versions: CamoVersions,
    compressed_spend_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    private_view: Scalar,
}
impl CamoViewKeysType1 {
    pub fn from_seed(
        view_seed: &SecretBytes<32>,
        master_spend: EdwardsPoint,
        i: u32,
        versions: CamoVersions,
    ) -> CamoViewKeysType1 {
        let (private_spend, private_view) = get_partial_keys(view_seed, i);
        let point_spend_key = master_spend + (private_spend * G);
        CamoViewKeysType1 {
            versions,
            compressed_spend_key: point_spend_key.compress(),
            point_spend_key,
            private_view,
        }
    }

    pub fn camo_versions(&self) -> CamoVersions {
        self.versions
    }

    pub fn to_camo_account(&self) -> CamoAccountType1 {
        points_to_account(self.versions, self.point_spend_key, &self.private_view * G)
    }

    pub fn signer_account(&self) -> Account {
        Account::from_both_points(&self.point_spend_key, &self.compressed_spend_key)
    }

    pub fn receiver_ecdh(&self, notification: &Notification) -> SecretBytes<32> {
        let point = match notification {
            Notification::V1(v1) => v1.representative_payload.point,
        };
        ecdh(&self.private_view, &point)
    }

    pub fn derive_account(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }
}

auto_from_impl!(From: CamoViewKeysType1 => SecretBytes<65>);
auto_from_impl!(TryFrom: SecretBytes<65> => CamoViewKeysType1);

impl From<&CamoViewKeysType1> for SecretBytes<65> {
    fn from(value: &CamoViewKeysType1) -> Self {
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
impl TryFrom<&SecretBytes<65>> for CamoViewKeysType1 {
    type Error = NanoError;

    fn try_from(value: &SecretBytes<65>) -> Result<Self, NanoError> {
        let bytes = value.as_ref();

        let versions = CamoVersions::decode_from_bits(bytes[0]);
        let compressed_spend_key = try_compressed_from_slice(&bytes[1..33])?;
        let point_spend_key = try_point_from_slice(&bytes[1..33])?;
        let private_view = Scalar::from_canonical_bytes(bytes[33..].as_ref().try_into().unwrap())?;

        Ok(CamoViewKeysType1 {
            versions,
            compressed_spend_key,
            point_spend_key,
            private_view,
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for CamoViewKeysType1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        CamoViewKeysType1Serde {
            versions: self.versions,
            point_spend_key: self.point_spend_key,
            private_view: self.private_view.clone(),
        }
        .serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CamoViewKeysType1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = CamoViewKeysType1Serde::deserialize(deserializer)?;
        Ok(CamoViewKeysType1 {
            versions: keys.versions,
            compressed_spend_key: keys.point_spend_key.compress(),
            point_spend_key: keys.point_spend_key,
            private_view: keys.private_view.clone(),
        })
    }
}
#[cfg(feature = "serde")]
#[derive(Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
struct CamoViewKeysType1Serde {
    versions: CamoVersions,
    point_spend_key: EdwardsPoint,
    private_view: Scalar,
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct CamoAccountType1 {
    account: String,
    versions: CamoVersions,
    compressed_spend_key: CompressedEdwardsY,
    compressed_view_key: CompressedEdwardsY,
    point_spend_key: EdwardsPoint,
    point_view_key: EdwardsPoint,
}
impl CamoAccountType1 {
    pub fn from_data(account: &str, data: &[u8]) -> Result<CamoAccountType1, NanoError> {
        account_from_data(account, data)
    }

    pub fn from_str(account: &str) -> Result<Self, NanoError> {
        let data = base32::decode(&account[CAMO_PREFIX_LEN..]).ok_or(NanoError::InvalidBase32)?;
        Self::from_data(account, &data)
    }

    pub fn camo_versions(&self) -> CamoVersions {
        self.versions
    }

    pub fn signer_account(&self) -> Account {
        Account::from_both_points(&self.point_spend_key, &self.compressed_spend_key)
    }

    pub fn sender_ecdh(
        &self,
        sender_key: &Key,
        sender_frontier: [u8; 32],
    ) -> (SecretBytes<32>, Notification) {
        let r = blake2b_scalar(
            &[
                sender_key.as_scalar().as_slice(),
                &sender_frontier,
                self.compressed_spend_key.as_bytes(),
            ]
            .concat(),
        );
        (ecdh(&r, &self.point_view_key), self.create_notification(&r))
    }

    fn create_notification(&self, r: &Scalar) -> Notification {
        let payload = r * G;
        match self.versions.highest_supported_version() {
            Some(CamoVersion::One) => Notification::create_v1(self.signer_account(), payload.into()),
            _ => panic!("broken CamoAccountType1 code: incompatible version accepted"),
        }
    }

    pub fn derive_account(&self, secret: &SecretBytes<32>, i: u32) -> Account {
        Account::from(self.point_spend_key + (get_account_scalar(secret, i) * G))
    }
}
impl Display for CamoAccountType1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.account)
    }
}
impl Hash for CamoAccountType1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.account.hash(state)
    }
}

#[cfg(feature = "serde")]
impl Serialize for CamoAccountType1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        CamoAccountType1Serde {
            versions: self.versions,
            point_spend_key: self.point_spend_key,
            point_view_key: self.point_view_key,
        }
        .serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CamoAccountType1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = CamoAccountType1Serde::deserialize(deserializer)?;
        Ok(points_to_account(
            keys.versions,
            keys.point_spend_key,
            keys.point_view_key,
        ))
    }
}
#[cfg(feature = "serde")]
#[derive(Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
struct CamoAccountType1Serde {
    versions: CamoVersions,
    point_spend_key: EdwardsPoint,
    point_view_key: EdwardsPoint,
}

camo_address_tests!(
    CamoKeysType1, CamoViewKeysType1, CamoAccountType1,
    versions!(1),
    "camo_18wydi3gmaw4aefwhkijrjw4qd87i4tc85wbnij95gz4em3qssickhpoj9i4t6taqk46wdnie7aj8ijrjhtcdgsp3c1oqnahct3otygxx4k7f3o4"
);
