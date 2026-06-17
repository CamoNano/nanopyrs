use crate::{auto_from_impl, Account, Block};
use std::hash::Hash;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A notification for a Camo transaction
#[repr(u8)]
#[derive(Debug, Clone, Hash, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Notification {
    /// Version 1-style notification (currently the only implemented version).
    V1(NotificationV1) = 1,
}
impl Notification {
    pub(crate) fn create_v1(recipient: Account, representative_payload: Account) -> Notification {
        Notification::V1(NotificationV1 {
            recipient,
            representative_payload,
        })
    }

    pub fn from_v1(block: &Block) -> Notification {
        Notification::V1(NotificationV1::from(block))
    }

    pub fn is_canonical(&self) -> bool {
        match self {
            Notification::V1(v1) => v1.is_canonical(),
        }
    }
}

/// Version 1-style notification (currently the only implemented version).
#[derive(Debug, Clone, Hash, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NotificationV1 {
    /// Send a small amount of Nano to this account.
    /// **Make that sure that the sender's representative is set to `representative_payload`**.
    ///
    /// Note that this account is publically linked to the camo account.
    pub recipient: Account,
    /// In the block sending to `notification_account`, make that sure this is set as the representative.
    /// This is the "payload" of the notification block.
    #[cfg_attr(feature = "serde", serde(rename = "payload"))]
    pub representative_payload: Account,
}
impl NotificationV1 {
    fn is_canonical(&self) -> bool {
        self.representative_payload.point.is_torsion_free()
    }
}
auto_from_impl!(From: Block => NotificationV1);
impl From<&Block> for NotificationV1 {
    fn from(value: &Block) -> Self {
        NotificationV1 {
            recipient: value.account.clone(),
            representative_payload: value.representative.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curve25519_dalek::constants::{ED25519_BASEPOINT_POINT as G, EIGHT_TORSION};
    use curve25519_dalek::Scalar;

    macro_rules! v1_with_payload {
        ($payload: expr) => {
            Notification::create_v1(Account::from(G), Account::from($payload)).is_canonical()
        };
    }

    #[test]
    fn canonical() {
        assert!(v1_with_payload!(G * Scalar::from(900_u32)));
        assert!(v1_with_payload!(G));
        assert!(v1_with_payload!(G + EIGHT_TORSION[0]));
    }

    #[test]
    fn non_canonical() {
        assert!(!v1_with_payload!(
            (G * Scalar::from(450_u32)) + EIGHT_TORSION[1]
        ));
        assert!(!v1_with_payload!(EIGHT_TORSION[5]));
        assert!(!v1_with_payload!(G + EIGHT_TORSION[6]));
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use crate::{constants::get_genesis_account, serde_test};

    serde_test!(notification_v1: NotificationV1 {
        recipient: get_genesis_account(),
        representative_payload: get_genesis_account()
    } => 32 + 32);

    serde_test!(notification: Notification::create_v1(get_genesis_account(), get_genesis_account()) => 4 + 64);
}
