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
    pub(crate) fn create_v1(
        notification_account: Account,
        representative_payload: Account,
    ) -> Notification {
        Notification::V1(NotificationV1 {
            notification_account,
            representative_payload,
        })
    }

    pub fn from_v1(block: Block)-> Notification {
        Notification::V1(NotificationV1::from(block))
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
    pub notification_account: Account,
    /// In the block sending to `notification_account`, make that sure this is set as the representative.
    /// This is the "payload" of the notification block.
    pub representative_payload: Account,
}
auto_from_impl!(From: Block => NotificationV1);
impl From<&Block> for NotificationV1 {
    fn from(value: &Block) -> Self {
        NotificationV1 {
            notification_account: value.account.clone(),
            representative_payload: value.representative.clone(),
        }
    }
}
