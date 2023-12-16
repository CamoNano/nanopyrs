use serde_json as json;
use json::{
    Value as JsonValue,
    Error as JsonError,
    Map
};
use reqwest::Error as ReqwestError;
use reqwest::{Client, RequestBuilder};
use thiserror::Error;
use std::num::ParseIntError;
use std::convert::From;
use crate::core::NanoError;
use hex::FromHexError;

pub(super) fn trim_json(value: String) -> String {
    value.trim_matches('\"').into()
}

#[derive(Debug, Error)]
pub enum RpcError {
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
    #[error(transparent)]
    JsonError(#[from] JsonError),
    #[error("parsing error: {0}")]
    ParseError(String),
    #[error("data was invalid")]
    InvalidData,
    #[error("RPC returned error: {0}")]
    ReturnedError(String),
    #[error("no rpc could be found for this action")]
    NoRPCs,
    #[error("this action could not be completed")]
    CommandFailed,
    #[error("cannot publish block of 'legacy' type")]
    LegacyBlockType
}
impl RpcError {
    pub fn from_option<T>(option: Option<T>) -> Result<T, RpcError> {
        option.ok_or(
            RpcError::ParseError("Option<T> returned empty".into())
        )
    }
}
impl From<ParseIntError> for RpcError {
    fn from(value: ParseIntError) -> Self {
        RpcError::ParseError(value.to_string())
    }
}
impl From<NanoError> for RpcError {
    fn from(value: NanoError) -> Self {
        RpcError::ParseError(value.to_string())
    }
}
impl From<FromHexError> for RpcError {
    fn from(value: FromHexError) -> Self {
        RpcError::ParseError(value.to_string())
    }
}

#[derive(Debug)]
pub struct InternalRpc {
    pub(crate) builder: RequestBuilder,
    pub(crate) url: String
}
impl InternalRpc {
    pub fn new(url: &str) -> InternalRpc {
        InternalRpc {
            builder: Client::new().post(url),
            url: url.into()
        }
    }

    pub async fn _raw_request(&self, json: JsonValue) -> Result<JsonValue, RpcError> {
        Ok(self.clone().builder
            .json(&json)
            .send().await?
            .json().await?
        )
    }

    pub async fn command(&self, command: String, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("action".into(), command.clone().into());

        let raw_json = self._raw_request(JsonValue::Object(arguments)).await?;
        if !raw_json["error"].is_null() {
            return Err(RpcError::ReturnedError(trim_json(raw_json["error"].to_string())))
        }
        Ok(raw_json)
    }


    pub async fn account_balance(&self, account: String, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("account".into(), account.into());
        self.command("account_balance".into(), arguments).await
    }

    pub async fn account_history(&self, account: String, count: usize, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("account".into(), account.into());
        arguments.insert("count".into(), count.to_string().into());
        self.command("account_history".into(), arguments).await
    }

    pub async fn accounts_balances(&self, accounts: &[String], arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("accounts".into(), accounts.into());
        self.command("accounts_balances".into(), arguments).await
    }

    pub async fn accounts_frontiers(&self, accounts: &[String]) -> Result<JsonValue, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("accounts".into(), accounts.into());
        self.command("accounts_frontiers".into(), arguments).await
    }

    pub async fn accounts_receivable(&self, accounts: &[String], count: usize, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("accounts".into(), accounts.into());
        arguments.insert("count".into(), count.into());
        self.command("accounts_receivable".into(), arguments).await
    }

    pub async fn accounts_representatives(&self, accounts: &[String]) -> Result<JsonValue, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("accounts".into(), accounts.into());
        self.command("accounts_representatives".into(), arguments).await
    }

    pub async fn block_info(&self, hash: String) -> Result<JsonValue, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("hash".into(), hash.into());
        arguments.insert("json_block".into(), "true".into());
        self.command("block_info".into(), arguments).await
    }

    pub async fn blocks_info(&self, hashes: &[String]) -> Result<JsonValue, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("hashes".into(), hashes.into());
        arguments.insert("json_block".into(), "true".into());
        self.command("blocks_info".into(), arguments).await
    }

    pub async fn process(&self, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("json_block".into(), "true".into());
        self.command("process".into(), arguments).await
    }

    pub async fn work_generate(&self, hash: String, arguments: Map<String, JsonValue>) -> Result<JsonValue, RpcError> {
        let mut arguments = arguments;
        arguments.insert("hash".into(), hash.into());
        self.command("work_generate".into(), arguments).await
    }
}
impl Clone for InternalRpc {
    fn clone(&self) -> Self {
        InternalRpc {
            builder: self.builder.try_clone().unwrap(),
            url: self.url.clone()
        }
    }
}