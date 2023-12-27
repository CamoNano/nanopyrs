use super::error::RpcError;
use serde_json as json;
use reqwest::{Client, RequestBuilder};
use json::{
    Value as JsonValue,
    Map
};

pub fn trim_json(value: String) -> String {
    value.trim_matches('\"').into()
}

#[derive(Debug)]
pub struct InternalRpc {
    pub(crate) builder: RequestBuilder,
    pub(crate) url: String
}
impl InternalRpc {
    pub fn new(url: &str, client: Client) -> Result<InternalRpc, RpcError> {
        Ok(InternalRpc {
            builder: client.post(url),
            url: url.into()
        })
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
        arguments.insert("use_peers".into(), "true".into());
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