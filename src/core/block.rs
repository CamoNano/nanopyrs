use super::nanopy::{sign_message, hash_block};
use super::constants::get_genesis_account;
use super::{Key, Account, NanoError, Signature};
use zeroize::Zeroize;
use std::fmt::Display;

pub use super::nanopy::{get_local_work, check_work};

#[derive(Debug, Clone, PartialEq, Zeroize)]
pub enum BlockType {
    Change,
    Send,
    Receive,
    Epoch,
    Legacy(String)
}
impl BlockType {
    pub fn is_state(&self) -> bool {
        !self.is_legacy()
    }
    pub fn is_legacy(&self) -> bool {
        matches!(self, BlockType::Legacy(_))
    }

    /// Only to be used for `state` blocks!
    pub fn from_subtype_string(value: &str) -> Option<BlockType> {
        match value {
            "change" => Some(BlockType::Change),
            "send" => Some(BlockType::Send),
            "receive" => Some(BlockType::Receive),
            "epoch" => Some(BlockType::Epoch),
            _ => None
        }
    }
}
impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str: String = match self {
            BlockType::Change => "change".into(),
            BlockType::Send => "send".into(),
            BlockType::Receive => "receive".into(),
            BlockType::Epoch => "epoch".into(),
            BlockType::Legacy(_type) => _type.into()
        };
        write!(f, "{}", as_str)
    }
}

#[derive(Debug, Clone, Zeroize)]
pub struct Block {
    pub block_type: BlockType,
    pub account: Account,
    pub previous: [u8; 32],
    pub representative: Account,
    pub balance: u128,
    pub link: [u8; 32],
    pub signature: Signature,
    pub work: [u8; 8]
}
impl Block {
    pub fn follows_epoch_rules(&self, previous: &Block) -> bool {
        self.balance == previous.balance &&
        self.representative == previous.representative
    }


    pub fn hash(&self) -> [u8; 32] {
        hash_block(self)
    }

    pub fn work_hash(&self) -> [u8; 32] {
        if self.previous == [0; 32] {
            self.account.compressed.to_bytes()
        } else {
            self.previous
        }
    }


    pub fn link_as_account(&self) -> Result<Account, NanoError> {
        Account::try_from(self.link)
    }


    pub fn get_signature(&self, private_key: &Key) -> Signature {
        sign_message(&self.hash(), private_key)
    }

    pub fn set_signature(&mut self, signature: Signature) {
        self.signature = signature
    }

    pub fn sign(&mut self, private_key: &Key) {
        self.set_signature(self.get_signature(private_key))
    }

    pub fn has_valid_signature(&self) -> bool {
        let signer = match self.block_type == BlockType::Epoch {
            true => get_genesis_account(),
            false => self.account.to_owned()
        };
        signer.is_valid_signature(&self.hash(), self.signature)
    }


    pub fn get_local_work(&self, difficulty: [u8; 8]) -> [u8; 8] {
        get_local_work(self.hash(), difficulty)
    }

    pub fn set_work(&mut self, work: [u8; 8]) {
        self.work = work
    }

    pub fn local_work(&mut self, work: [u8; 8]) {
        self.work = self.get_local_work(work)
    }

    pub fn has_valid_work(&self, difficulty: [u8; 8]) -> bool {
        check_work(self.hash(), difficulty, self.work)
    }
}


#[cfg(test)]
mod tests {
    use crate::{Key, SecretBytes, constants::ONE_NANO};
    use super::*;

    const TEST_WORK_DIFFICULTY: [u8; 8] = 0xfff8000000000000_u64.to_be_bytes();

    fn create_test_block() -> Block {
        let seed = SecretBytes::from(&mut [0; 32]);
        let key = Key::from_seed(&seed, 0);
        let account = key.to_account();
        let representative = Key::from_seed(&seed, 1).to_account();

        Block {
            block_type: BlockType::Send,
            account,
            previous: [127; 32],
            representative,
            balance: ONE_NANO,
            link: [128; 32],

            signature: [0; 64].try_into().unwrap(),
            work: [0; 8]
        }
    }

    #[test]
    fn work() {
        let mut block = create_test_block();

        assert!(!block.has_valid_work([255; 8]));
        block.local_work(TEST_WORK_DIFFICULTY);
        assert!(block.has_valid_work(TEST_WORK_DIFFICULTY));
    }

    #[test]
    fn signature() {
        let seed = SecretBytes::from(&mut [0; 32]);
        let key = Key::from_seed(&seed, 0);
        let mut block = create_test_block();

        assert!(!block.has_valid_signature());
        block.sign(&key);
        assert!(block.has_valid_signature());
    }
}