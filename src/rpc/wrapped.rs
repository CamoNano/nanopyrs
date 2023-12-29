use crate::{Account, Block};
use super::{parse, request};
use super::util::trim_json;
use super::error::RpcError;
use reqwest::{ClientBuilder, RequestBuilder, Proxy};
use serde_json as json;
use json::{
    Value as JsonValue,
    Map
};

/// See the official [Nano RPC documentation](https://docs.nano.org/commands/rpc-protocol/) for details.
#[derive(Debug)]
pub struct Rpc {
    builder: RequestBuilder,
    url: String,
    proxy: Option<String>
}
impl Rpc {
    pub fn new(url: &str) -> Result<Rpc, RpcError> {
        Ok(Rpc {
            builder: ClientBuilder::new().build()?.post(url),
            url: url.into(),
            proxy: None
        })
    }

    pub fn new_with_proxy(url: &str, proxy: &str) -> Result<Rpc, RpcError> {
        Ok(Rpc {
            builder: ClientBuilder::new().proxy(Proxy::all(proxy)?).build()?.post(url),
            url: url.into(),
            proxy: Some(proxy.into())
        })
    }

    /// Get the url of this RPC
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    /// Same as `command`, but *everything* must be set manually
    pub async fn _raw_request(&self, json: JsonValue) -> Result<JsonValue, RpcError> {
        Ok(self.clone().builder
            .json(&json)
            .send().await?
            .json().await?
        )
    }

    /// Send a request to the node with `action` set to `[command]`, and setting the given `arguments`
    pub async fn command(&self, command: &str, mut arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        arguments.insert("action".into(), command.clone().into());

        let raw_json = self._raw_request(JsonValue::Object(arguments)).await?;
        if !raw_json["error"].is_null() {
            return Err(RpcError::ReturnedError(trim_json(raw_json["error"].to_string())))
        }
        Ok(raw_json)
    }


    pub async fn account_balance(&self, account: &Account) -> Result<u128, RpcError> {
        parse::account_balance(
            request::account_balance(self, account).await?
        ).await
    }

    /// Lists the account's blocks, starting at `head` (or the newest block if `head` is `None`), and going back at most `count` number of blocks.
    /// Will stop at first legacy block.
    pub async fn account_history(&self, account: &Account, count: usize, head: Option<[u8; 32]>) -> Result<Vec<Block>, RpcError> {
        parse::account_history(
            request::account_history(self, account, count, head).await?, account
        ).await
    }

    /// Indirect, relies on `account_history`. This allows the data to be verified to an extent.
    pub async fn account_representative(&self, account: &Account) -> Result<Account, RpcError> {
        parse::account_representative(
            self.account_history(account, 1, None).await?
        ).await
    }

    pub async fn accounts_balances(&self, accounts: &[Account]) -> Result<Vec<u128>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }
        parse::accounts_balances(
            request::accounts_balances(self, accounts).await?, accounts
        ).await
    }

    /// Returns the hash of the frontier (newest) block of the given accounts.
    /// If an account is not yet opened, its frontier will be returned as `[0; 32]`
    pub async fn accounts_frontiers(&self, accounts: &[Account]) -> Result<Vec<[u8; 32]>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }
        parse::accounts_frontiers(
            request::accounts_frontiers(self, accounts).await?, accounts
        ).await
    }

    /// For each account, returns the receivable transactions as `Vec<(block_hash, amount)>`
    pub async fn accounts_receivable(&self, accounts: &[Account], count: usize, threshold: u128) -> Result<Vec<Vec<([u8; 32], u128)>>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }
        parse::accounts_receivable(
            request::accounts_receivable(self, accounts, count, threshold).await?, accounts
        ).await
    }

    /// If an account is not yet opened, its frontier will be returned as `None`
    pub async fn accounts_representatives(&self, accounts: &[Account]) -> Result<Vec<Option<Account>>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }
        parse::accounts_representatives(
            request::accounts_representatives(self, accounts).await?, accounts
        ).await
    }

    /// Legacy blocks will return `None`
    pub async fn block_info(&self, hash: [u8; 32]) -> Result<Option<Block>, RpcError> {
        parse::block_info(
            request::block_info(self, hash).await?
        ).await
    }

    /// Legacy blocks will return `None`
    pub async fn blocks_info(&self, hashes: &[[u8; 32]]) -> Result<Vec<Option<Block>>, RpcError> {
        if hashes.is_empty() {
            return Ok(vec!());
        }
        parse::blocks_info(
            request::blocks_info(self, hashes).await?, hashes
        ).await
    }

    /// Returns the hash of the block
    pub async fn process(&self, block: Block) -> Result<[u8; 32], RpcError> {
        let hash = block.hash();
        parse::process(
            request::process(self, block).await?["blocks"].clone(), hash
        ).await
    }

    /// Returns the generated work, assuming no error is encountered
    pub async fn work_generate(&self, work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> Result<[u8; 8], RpcError> {
        parse::work_generate(
            request::work_generate(self, work_hash, custom_difficulty).await?, work_hash, custom_difficulty
        ).await
    }
}
impl Clone for Rpc {
    fn clone(&self) -> Self {
        Rpc {
            builder: self.builder.try_clone().unwrap(),
            url: self.url.clone(),
            proxy: self.proxy.clone()
        }
    }
}