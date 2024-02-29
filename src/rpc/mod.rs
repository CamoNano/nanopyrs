mod encode;
mod error;
mod parse;

pub mod debug;
pub mod util;

use crate::{Account, Block};
use debug::DebugRpc;
use json::{Map, Value as JsonValue};
use serde_json as json;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use error::RpcError;

/// A receivable (pending) transaction.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AccountInfo {
    pub frontier: [u8; 32],
    pub open_block: [u8; 32],
    pub representative_block: [u8; 32],
    pub balance: u128,
    pub modified_timestamp: u64,
    pub block_count: usize,
    pub version: usize,
    pub representative: Account,
    pub weight: u128,
    pub receivable: usize,
}

/// A receivable (pending) transaction.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Receivable {
    /// The recipient account of this transaction
    pub recipient: Account,
    /// The hash of the send block on the sender's account
    pub block_hash: [u8; 32],
    /// The amount being transferred
    pub amount: u128,
}
impl From<(Account, [u8; 32], u128)> for Receivable {
    fn from(value: (Account, [u8; 32], u128)) -> Self {
        Receivable {
            recipient: value.0,
            block_hash: value.1,
            amount: value.2,
        }
    }
}

/// See the official [Nano RPC documentation](https://docs.nano.org/commands/rpc-protocol/) for details.
#[derive(Debug, Clone)]
pub struct Rpc(DebugRpc);
impl Rpc {
    pub fn new(url: &str) -> Result<Rpc, RpcError> {
        Ok(Rpc(DebugRpc::new(url)?))
    }

    pub fn new_with_proxy(url: &str, proxy: &str) -> Result<Rpc, RpcError> {
        Ok(Rpc(DebugRpc::new_with_proxy(url, proxy)?))
    }

    /// Get the URL of this RPC
    pub fn get_url(&self) -> &str {
        self.0.get_url()
    }

    /// Get the proxy of this RPC, if set
    pub fn get_proxy(&self) -> Option<&str> {
        self.0.get_proxy()
    }

    /// Same as `command`, but *everything* must be set manually
    pub async fn _raw_request(&self, json: JsonValue) -> Result<JsonValue, RpcError> {
        self.0._raw_request(json).await.result
    }

    /// Send a request to the node with `action` set to `[command]`, and setting the given `arguments`
    pub async fn command(
        &self,
        command: &str,
        arguments: Map<String, JsonValue>,
    ) -> Result<JsonValue, RpcError> {
        self.0.command(command, arguments).await.result
    }

    pub async fn account_balance(&self, account: &Account) -> Result<u128, RpcError> {
        self.0.account_balance(account).await.result
    }

    /// Lists the account's blocks, starting at `head` (or the newest block if `head` is `None`), and going back at most `count` number of blocks.
    /// Will stop at first legacy block.
    pub async fn account_history(
        &self,
        account: &Account,
        count: usize,
        head: Option<[u8; 32]>,
        offset: Option<usize>,
    ) -> Result<Vec<Block>, RpcError> {
        self.0
            .account_history(account, count, head, offset)
            .await
            .result
    }

    /// Gets general information about an account
    pub async fn account_info(&self, account: &Account) -> Result<AccountInfo, RpcError> {
        self.0.account_info(account).await.result
    }

    /// Indirect, relies on `account_history`. This allows the data to be verified to an extent.
    pub async fn account_representative(&self, account: &Account) -> Result<Account, RpcError> {
        self.0.account_representative(account).await.result
    }

    pub async fn accounts_balances(&self, accounts: &[Account]) -> Result<Vec<u128>, RpcError> {
        self.0.accounts_balances(accounts).await.result
    }

    /// Returns the hash of the frontier (newest) block of the given accounts.
    /// If an account is not yet opened, its frontier will be returned as `None`.
    pub async fn accounts_frontiers(
        &self,
        accounts: &[Account],
    ) -> Result<Vec<Option<[u8; 32]>>, RpcError> {
        self.0.accounts_frontiers(accounts).await.result
    }

    /// For each account, returns the receivable transactions as `Vec<Receivable>`
    pub async fn accounts_receivable(
        &self,
        accounts: &[Account],
        count: usize,
        threshold: u128,
    ) -> Result<Vec<Vec<Receivable>>, RpcError> {
        self.0
            .accounts_receivable(accounts, count, threshold)
            .await
            .result
    }

    /// If an account is not yet opened, its frontier will be returned as `None`
    pub async fn accounts_representatives(
        &self,
        accounts: &[Account],
    ) -> Result<Vec<Option<Account>>, RpcError> {
        self.0.accounts_representatives(accounts).await.result
    }

    /// Legacy blocks will return `None`
    pub async fn block_info(&self, hash: [u8; 32]) -> Result<Option<Block>, RpcError> {
        self.0.block_info(hash).await.result
    }

    /// Legacy blocks will return `None`
    pub async fn blocks_info(&self, hashes: &[[u8; 32]]) -> Result<Vec<Option<Block>>, RpcError> {
        self.0.blocks_info(hashes).await.result
    }

    /// Returns the hash of the block
    pub async fn process(&self, block: &Block) -> Result<[u8; 32], RpcError> {
        self.0.process(block).await.result
    }

    /// Returns the generated work, assuming no error is encountered
    pub async fn work_generate(
        &self,
        work_hash: [u8; 32],
        custom_difficulty: Option<[u8; 8]>,
    ) -> Result<[u8; 8], RpcError> {
        self.0
            .work_generate(work_hash, custom_difficulty)
            .await
            .result
    }
}
