use crate::{Account, Block};
use super::internal::InternalRpc;
use super::parse;
use super::util::block_to_json;
use super::error::RpcError;
use reqwest::{ClientBuilder, Proxy};
use serde_json as json;
use json::{
    Value as JsonValue,
    Map
};

/// See Nano's [RPC documentation](https://docs.nano.org/commands/rpc-protocol/) for details.
#[derive(Debug, Clone)]
pub struct Rpc {
    internal: InternalRpc
}
impl Rpc {
    pub fn new(url: &str) -> Result<Rpc, RpcError> {
        let client = ClientBuilder::new()
            .build()?;
        Ok(Rpc { internal: InternalRpc::new(url, client)? })
    }

    pub fn new_with_proxy(url: &str, proxy: &str) -> Result<Rpc, RpcError> {
        let client = ClientBuilder::new()
            .proxy(Proxy::all(proxy)?)
            .build()?;
        Ok(Rpc { internal: InternalRpc::new(url, client)? })
    }

    pub fn get_url(&self) -> String {
        self.internal.url.clone()
    }

    pub async fn _raw_request(&self, json: JsonValue) -> Result<JsonValue, RpcError> {
        self.internal._raw_request(json).await
    }

    pub async fn command(&self, command: &str, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        self.internal.command(command.to_owned(), arguments).await
    }


    pub async fn account_balance(&self, account: &Account) -> Result<u128, RpcError> {
        let raw_json = self.internal.account_balance(account.to_string(), Map::new()).await?;

        parse::account_balance(raw_json).await
    }

    /// Will stop at first legacy block
    pub async fn account_history(&self, account: &Account, count: usize, head: Option<[u8; 32]>) -> Result<Vec<Block>, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("raw".into(), "true".into());
        if let Some(head) = head {
            arguments.insert("head".into(), hex::encode(head).into());
        }

        let raw_json = self.internal
            .account_history(account.to_string(), count, arguments).await?;

        parse::account_history(raw_json, account).await
    }

    /// Indirect, relies on account_history
    pub async fn account_representative(&self, account: &Account) -> Result<Account, RpcError> {
        let history = self.account_history(account, 1, None).await?;

        parse::account_representative(history).await
    }

    pub async fn accounts_balances(&self, accounts: &[Account]) -> Result<Vec<u128>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();
        let raw_json = self.internal.accounts_balances(&accounts, Map::new()).await?;

        parse::accounts_balances(raw_json, accounts).await
    }

    pub async fn accounts_frontiers(&self, accounts: &[Account]) -> Result<Vec<[u8; 32]>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();
        let raw_json = self.internal.accounts_frontiers(&accounts)
            .await?["frontiers"].clone();

        parse::accounts_frontiers(raw_json, accounts).await
    }

    pub async fn accounts_receivable(&self, accounts: &[Account], count: usize, threshold: u128) -> Result<Vec<Vec<([u8; 32], u128)>>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let mut arguments = Map::new();
        arguments.insert("sorting".into(), "true".into());
        arguments.insert("threshold".into(), threshold.to_string().into());

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();
        let raw_json = self.internal
            .accounts_receivable(&accounts, count, arguments).await?["blocks"]
            .clone();

        parse::accounts_receivable(raw_json, accounts).await
    }

    pub async fn accounts_representatives(&self, accounts: &[Account]) -> Result<Vec<Option<Account>>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }
        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();
        let raw_json = self.internal.accounts_representatives(&accounts).await?["representatives"].clone();

        parse::accounts_representatives(raw_json, accounts).await
    }

    /// Legacy blocks will return `None`
    pub async fn block_info(&self, hash: [u8; 32]) -> Result<Option<Block>, RpcError> {
        let raw_json = self.internal.block_info(hex::encode(hash)).await?;

        parse::block_info(raw_json).await
    }

    /// Legacy blocks will return `None`
    pub async fn blocks_info(&self, hashes: &[[u8; 32]]) -> Result<Vec<Option<Block>>, RpcError> {
        if hashes.is_empty() {
            return Ok(vec!());
        }
        let hashes: Vec<String> = hashes.iter()
            .map(hex::encode).collect();
        let raw_json = self.internal.blocks_info(&hashes).await?["blocks"].clone();

        parse::blocks_info(raw_json, hashes).await
    }

    pub async fn process(&self, block: Block) -> Result<[u8; 32], RpcError> {
        let mut arguments = Map::new();
        let hash = block.hash();

        if !block.block_type.is_state() {
            return Err(RpcError::LegacyBlockType)
        }
        arguments.insert("subtype".into(), block.block_type.to_string().into());
        arguments.insert("block".into(), JsonValue::Object(block_to_json(block)));

        let raw_json = self.internal.process(arguments).await?;
        parse::process(raw_json, hash).await
    }

    pub async fn work_generate(&self, work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> Result<[u8; 8], RpcError> {
        let mut map = Map::new();
        if let Some(difficulty) = custom_difficulty {
            map.insert("difficulty".into(), hex::encode(difficulty).into());
        }
        let raw_json = self.internal.work_generate(hex::encode(work_hash), map).await?;

        parse::work_generate(raw_json, work_hash, custom_difficulty).await
    }
}