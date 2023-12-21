use super::nanopy::{sign_message, hash_block};
use super::constants::{get_genesis_account, epoch_signers::*};
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
        if self.block_type != BlockType::Epoch {
            // "normal" block
            self.account.to_owned()

        } else if self.link[7] == 49 {
            // epoch v1
            get_v1_epoch_signer()

        } else if self.link[7] == 50 {
            // epoch v2
            get_v2_epoch_signer()

        } else {
            // "uhhh let's try genesis I guess"
            get_genesis_account()

        }.is_valid_signature(&self.hash(), self.signature)
    }


    pub fn get_local_work(&self, difficulty: [u8; 8]) -> [u8; 8] {
        get_local_work(self.work_hash(), difficulty)
    }

    pub fn set_work(&mut self, work: [u8; 8]) {
        self.work = work
    }

    pub fn local_work(&mut self, work: [u8; 8]) {
        self.work = self.get_local_work(work)
    }

    pub fn has_valid_work(&self, difficulty: [u8; 8]) -> bool {
        if self.block_type == BlockType::Epoch {
            return true
        }
        check_work(self.work_hash(), difficulty, self.work)
    }
}


#[cfg(test)]
mod tests {
    use crate::{Key, SecretBytes, constants::ONE_NANO};
    use super::*;

    const TEST_WORK_DIFFICULTY: [u8; 8] = 0xfff8000000000000_u64.to_be_bytes();
    const NORMAL_WORK_DIFFICULTY: [u8; 8] = 0xfffffff800000000_u64.to_be_bytes();
    const INFINITE_WORK_DIFFICULTY: [u8; 8] = 0xffffffffffffffff_u64.to_be_bytes();

    fn create_test_block() -> Block {
        let seed = SecretBytes::from([0; 32]);
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

            signature: Signature::default(),
            work: [0; 8]
        }
    }

    #[test]
    fn create_work() {
        let mut block = create_test_block();

        assert!(!block.has_valid_work([255; 8]));
        block.local_work(TEST_WORK_DIFFICULTY);
        assert!(block.has_valid_work(TEST_WORK_DIFFICULTY));
    }

    #[test]
    fn create_signature() {
        let seed = SecretBytes::from([0; 32]);
        let key = Key::from_seed(&seed, 0);
        let mut block = create_test_block();

        assert!(!block.has_valid_signature());
        block.sign(&key);
        assert!(block.has_valid_signature());
    }

    #[test]
    fn check_receive_block() {
        let block = Block {
            block_type: BlockType::Receive,
            account: Account::try_from("nano_3cpz7oh9qr5b7obbcb5867omqf8esix4sdd5w6mh8kkknamjgbnwrimxsaaf").unwrap(),
            previous: [
                129, 149, 239, 153, 243, 86, 55, 9, 146, 47, 120, 27, 208, 150, 213, 51,
                143, 223, 27, 91, 132, 108, 97, 183, 154, 231, 115, 156, 215, 69, 70, 191
            ],
            representative: Account::try_from("nano_37imps4zk1dfahkqweqa91xpysacb7scqxf3jqhktepeofcxqnpx531b3mnt").unwrap(),
            balance: 12603866388773874271376430197004955478,
            link: [
                193, 250, 200, 172, 202, 201, 47, 111, 83, 111, 26, 144, 241, 161, 185, 32,
                122, 213, 135, 172, 79, 45, 4, 159, 94, 138, 37, 188, 78, 58, 33, 165
            ],
            signature: Signature::try_from([
                26, 22, 203, 145, 161, 117, 150, 35, 205, 5, 230, 39, 56, 46, 120, 162,
                109, 124, 117, 80, 239, 18, 102, 1, 221, 148, 13, 79, 185, 74, 136, 50,
                120, 216, 236, 159, 181, 147, 184, 247, 25, 54, 51, 130, 242, 12, 58, 52,
                182, 38, 180, 138, 157, 195, 109, 244, 41, 5, 7, 40, 92, 87, 158, 6
            ]).unwrap(),
            work: [55, 16, 153, 165, 103, 12, 179, 237]
        };
        assert!(block.has_valid_work(NORMAL_WORK_DIFFICULTY));
        assert!(block.has_valid_signature());
    }

    #[test]
    fn check_send_block() {
        let block = Block {
            block_type: BlockType::Send,
            account: Account::try_from("nano_3cpz7oh9qr5b7obbcb5867omqf8esix4sdd5w6mh8kkknamjgbnwrimxsaaf").unwrap(),
            previous: [
                51, 190, 253, 128, 226, 21, 179, 253, 60, 46, 69, 62, 113, 112, 141, 197,
                34, 189, 51, 236, 38, 152, 45, 3, 139, 137, 116, 69, 182, 168, 248, 216
            ],
            representative: Account::try_from("nano_37imps4zk1dfahkqweqa91xpysacb7scqxf3jqhktepeofcxqnpx531b3mnt").unwrap(),
            balance: 12603714974808874271376430197004955478,
            link: [
                143, 164, 224, 238, 131, 161, 166, 194, 112, 31, 106, 114, 154, 181, 0, 254,
                225, 165, 19, 125, 57, 54, 49, 25, 11, 249, 132, 155, 203, 219, 197, 162
            ],
            signature: Signature::try_from([
                231, 93, 74, 12, 164, 163, 118, 237, 82, 31, 44, 126, 192, 173, 115, 218,
                185, 6, 59, 18, 168, 143, 202, 222, 231, 162, 27, 192, 186, 117, 165, 3,
                83, 254, 199, 11, 204, 25, 25, 162, 248, 234, 125, 30, 174, 248, 143, 13,
                196, 210, 136, 200, 7, 193, 239, 62, 51, 131, 230, 67, 137, 89, 150, 7
            ]).unwrap(),
            work: [13, 162, 2, 90, 186, 82, 152, 241]
        };
        assert!(block.has_valid_work(NORMAL_WORK_DIFFICULTY));
        assert!(block.has_valid_signature());
    }

    #[test]
    fn check_epoch_v1() {
        let block = Block {
            block_type: BlockType::Epoch,
            account: Account::try_from("nano_35jjmmmh81kydepzeuf9oec8hzkay7msr6yxagzxpcht7thwa5bus5tomgz9").unwrap(),
            previous: [
                197, 41, 171, 147, 162, 137, 248, 248, 155, 150, 79, 76, 151, 13, 151, 82,
                8, 154, 65, 86, 228, 196, 79, 112, 118, 20, 73, 181, 151, 153, 123, 223
            ],
            representative: Account::try_from("nano_3arg3asgtigae3xckabaaewkx3bzsh7nwz7jkmjos79ihyaxwphhm6qgjps4").unwrap(),
            balance: 795055344175165130955846320127,
            link: [
                101, 112, 111, 99, 104, 32, 118, 49, 32, 98, 108, 111, 99, 107, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            signature: Signature::try_from([
                52, 10, 149, 153, 90, 136, 154, 249, 218, 117, 203, 27, 150, 230, 130, 245,
                72, 66, 102, 174, 174, 72, 56, 20, 52, 67, 230, 176, 167, 160, 140, 135,
                105, 137, 83, 44, 117, 7, 96, 241, 31, 213, 191, 12, 82, 173, 120, 237,
                118, 22, 139, 159, 153, 184, 216, 4, 50, 101, 206, 107, 55, 165, 79, 6
            ]).unwrap(),
            work: [133, 203, 130, 102, 22, 143, 154, 3]
        };
        assert!(block.has_valid_work(INFINITE_WORK_DIFFICULTY));
        assert!(block.has_valid_signature());
    }

    #[test]
    fn check_epoch_v2() {
        let block = Block {
            block_type: BlockType::Epoch,
            account: Account::try_from("nano_35jjmmmh81kydepzeuf9oec8hzkay7msr6yxagzxpcht7thwa5bus5tomgz9").unwrap(),
            previous: [
                95, 36, 90, 242, 101, 15, 47, 82, 125, 66, 179, 207, 122, 91, 39, 142,
                2, 82, 218, 93, 89, 147, 120, 8, 194, 142, 100, 112, 195, 173, 251, 41
            ],
            representative: Account::try_from("nano_3arg3asgtigae3xckabaaewkx3bzsh7nwz7jkmjos79ihyaxwphhm6qgjps4").unwrap(),
            balance: 795055344175165130955846320127,
            link: [
                101, 112, 111, 99, 104, 32, 118, 50, 32, 98, 108, 111, 99, 107, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            signature: Signature::try_from([
                245, 214, 91, 76, 153, 189, 130, 100, 140, 166, 131, 115, 32, 218, 225, 204,
                49, 222, 162, 246, 59, 194, 18, 139, 98, 240, 1, 1, 133, 84, 221, 168,
                26, 177, 21, 118, 213, 138, 29, 191, 105, 72, 109, 16, 225, 29, 45, 67,
                241, 49, 197, 181, 71, 70, 70, 2, 100, 196, 90, 52, 22, 71, 158, 4
            ]).unwrap(),
            work: [178, 49, 190, 86, 245, 226, 43, 160]
        };
        assert!(block.has_valid_work(INFINITE_WORK_DIFFICULTY));
        assert!(block.has_valid_signature());
    }
}