use super::{encode, error::RpcError, parse, AccountInfo, BlockInfo, Receivable};
use crate::{Account, Block};

use json::{Map, Value as JsonValue};
use reqwest::{ClientBuilder, Proxy, RequestBuilder};
use serde_json as json;

macro_rules! request {
    ($rpc: expr, $json: expr) => {
        $rpc._raw_request($json).await
    };
}

macro_rules! map_response {
    ($response: expr, $new_result: expr) => {
        Response {
            raw_request: $response.raw_request,
            raw_response: $response.raw_response,
            result: $new_result,
        }
    };
}

#[derive(Debug)]
pub struct Response<T> {
    pub raw_request: Option<JsonValue>,
    pub raw_response: Option<JsonValue>,
    pub result: Result<T, RpcError>,
}
impl<T> Response<T> {
    fn no_request(result: Result<T, RpcError>) -> Response<T> {
        Response {
            raw_request: None,
            raw_response: None,
            result,
        }
    }
}

/// See the official [Nano RPC documentation](https://docs.nano.org/commands/rpc-protocol/) for details.
#[derive(Debug)]
pub struct DebugRpc {
    builder: RequestBuilder,
    url: String,
    proxy: Option<String>,
}
impl DebugRpc {
    pub fn new(url: &str) -> Result<DebugRpc, RpcError> {
        Ok(DebugRpc {
            builder: ClientBuilder::new().build()?.post(url),
            url: url.into(),
            proxy: None,
        })
    }

    pub fn new_with_proxy(url: &str, proxy: &str) -> Result<DebugRpc, RpcError> {
        Ok(DebugRpc {
            builder: ClientBuilder::new()
                .proxy(Proxy::all(proxy)?)
                .build()?
                .post(url),
            url: url.into(),
            proxy: Some(proxy.into()),
        })
    }

    /// Get the url of this RPC
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Get the proxy of this RPC, if set
    pub fn get_proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    /// Same as `command`, but *everything* must be set manually
    pub async fn _raw_request(&self, json: JsonValue) -> Response<JsonValue> {
        let response_json = self
            .clone()
            .builder
            .json(&json)
            .send()
            .await
            .map_err(RpcError::ReqwestError)
            .map(|response| response.json::<JsonValue>());

        let result = match response_json {
            Ok(response) => response.await.map_err(RpcError::ReqwestError),
            Err(err) => Err(err),
        };

        let raw_response = match &result {
            Ok(json) => Some(json.clone()),
            Err(_) => None,
        };

        Response {
            raw_request: Some(json),
            raw_response,
            result,
        }
    }

    /// Send a request to the node with `action` set to `[command]`, and setting the given `arguments`
    pub async fn command(
        &self,
        command: &str,
        mut arguments: Map<String, JsonValue>,
    ) -> Response<JsonValue> {
        arguments.insert("action".into(), command.into());
        self._raw_request(JsonValue::Object(arguments)).await
    }

    pub async fn account_balance(&self, account: &Account) -> Response<u128> {
        let response = request!(self, encode::account_balance(account));
        let result = match response.result {
            Ok(json) => parse::account_balance(json),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Lists the account's blocks, starting at `head` (or the newest block if `head` is `None`), and going back at most `count` number of blocks.
    /// Will stop at first legacy block.
    pub async fn account_history(
        &self,
        account: &Account,
        count: usize,
        head: Option<[u8; 32]>,
        offset: Option<usize>,
    ) -> Response<Vec<Block>> {
        let response = request!(self, encode::account_history(account, count, head, offset));
        let result = match response.result {
            Ok(json) => parse::account_history(json, account),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Gets general information about an account.
    /// Returns `None` if the account has not been opened.
    pub async fn account_info(&self, account: &Account) -> Response<Option<AccountInfo>> {
        let response = request!(self, encode::account_info(account));
        let result = match response.result {
            Ok(json) => parse::account_info(json),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Indirect, relies on `account_history`.
    /// This allows the data to be verified to an extent.
    ///
    /// If an account is not yet opened, its representative will be returned as `None`.
    pub async fn account_representative(&self, account: &Account) -> Response<Option<Account>> {
        let response = self.account_history(account, 1, None, None).await;
        let result = match response.result {
            Ok(history) => parse::account_representative(history),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    pub async fn accounts_balances(&self, accounts: &[Account]) -> Response<Vec<u128>> {
        if accounts.is_empty() {
            return Response::no_request(Ok(vec![]));
        }

        let response = request!(self, encode::accounts_balances(accounts));
        let result = match response.result {
            Ok(json) => parse::accounts_balances(json, accounts),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Returns the hash of the frontier (newest) block of the given accounts.
    /// If an account is not yet opened, its frontier will be returned as `None`.
    pub async fn accounts_frontiers(
        &self,
        accounts: &[Account],
    ) -> Response<Vec<Option<[u8; 32]>>> {
        if accounts.is_empty() {
            return Response::no_request(Ok(vec![]));
        }

        let response = request!(self, encode::accounts_frontiers(accounts));
        let result = match response.result {
            Ok(json) => parse::accounts_frontiers(json, accounts),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// For each account, returns the receivable transactions as `Vec<Receivable>`
    pub async fn accounts_receivable(
        &self,
        accounts: &[Account],
        count: usize,
        threshold: u128,
    ) -> Response<Vec<Vec<Receivable>>> {
        if accounts.is_empty() {
            return Response::no_request(Ok(vec![]));
        }

        let response = request!(
            self,
            encode::accounts_receivable(accounts, count, threshold)
        );
        let result = match response.result {
            Ok(json) => parse::accounts_receivable(json, accounts),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// If an account is not yet opened, its representative will be returned as `None`
    pub async fn accounts_representatives(
        &self,
        accounts: &[Account],
    ) -> Response<Vec<Option<Account>>> {
        if accounts.is_empty() {
            return Response::no_request(Ok(vec![]));
        }

        let response = request!(self, encode::accounts_representatives(accounts));
        let result = match response.result {
            Ok(json) => parse::accounts_representatives(json, accounts),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Legacy blocks, and blocks that don't exist, will return `None`
    pub async fn block_info(&self, hash: [u8; 32]) -> Response<Option<BlockInfo>> {
        let response = request!(self, encode::block_info(hash));
        let result = match response.result {
            Ok(json) => parse::block_info(json),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Legacy blocks, and blocks that don't exist, will return `None`
    pub async fn blocks_info(&self, hashes: &[[u8; 32]]) -> Response<Vec<Option<BlockInfo>>> {
        if hashes.is_empty() {
            return Response::no_request(Ok(vec![]));
        }

        let response = request!(self, encode::blocks_info(hashes));
        let result = match response.result {
            Ok(json) => parse::blocks_info(json, hashes),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Returns the hash of the block
    pub async fn process(&self, block: &Block) -> Response<[u8; 32]> {
        if !block.block_type.is_state() {
            return Response::no_request(Err(RpcError::LegacyBlockType));
        }

        let hash = block.hash();
        let response = request!(self, encode::process(block));
        let result = match response.result {
            Ok(json) => parse::process(json, hash),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }

    /// Returns the generated work, assuming no error is encountered
    pub async fn work_generate(
        &self,
        work_hash: [u8; 32],
        custom_difficulty: Option<[u8; 8]>,
    ) -> Response<[u8; 8]> {
        let response = request!(self, encode::work_generate(work_hash, custom_difficulty));
        let result = match response.result {
            Ok(json) => parse::work_generate(json, work_hash, custom_difficulty),
            Err(err) => Err(err),
        };
        map_response!(response, result)
    }
}
impl Clone for DebugRpc {
    fn clone(&self) -> Self {
        DebugRpc {
            builder: self.builder.try_clone().unwrap(),
            url: self.url.clone(),
            proxy: self.proxy.clone(),
        }
    }
}
