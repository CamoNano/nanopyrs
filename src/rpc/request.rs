use crate::{Account, Block};
use super::Rpc;
use super::util::block_to_json;
use super::error::RpcError;
use serde_json as json;
use json::{
    Value as JsonValue,
    Map
};


pub async fn account_balance(rpc: &Rpc, account: &Account) -> Result<JsonValue, RpcError> {
    let mut arguments = Map::new();
    arguments.insert("account".into(), account.into());

    rpc.command("account_balance".into(), arguments).await
}

pub async fn account_history(rpc: &Rpc, account: &Account, count: usize, head: Option<[u8; 32]>) -> Result<JsonValue, RpcError> {
    let mut arguments = Map::new();
    arguments.insert("raw".into(), "true".into());
    arguments.insert("account".into(), account.into());
    arguments.insert("count".into(), count.to_string().into());
    if let Some(head) = head {
        arguments.insert("head".into(), hex::encode(head).into());
    }

    rpc.command("account_history".into(), arguments).await
}

pub async fn accounts_balances(rpc: &Rpc, accounts: &[Account]) -> Result<JsonValue, RpcError> {
    let accounts: Vec<String> = accounts.iter()
        .map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("accounts".into(), accounts.as_slice().into());

    rpc.command("accounts_balances".into(), arguments).await
}

pub async fn accounts_frontiers(rpc: &Rpc, accounts: &[Account]) -> Result<JsonValue, RpcError> {
    let accounts: Vec<String> = accounts.iter()
        .map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("accounts".into(), accounts.as_slice().into());

    Ok(rpc.command("accounts_frontiers".into(), arguments)
        .await?["frontiers"].clone())
}

pub async fn accounts_receivable(rpc: &Rpc, accounts: &[Account], count: usize, threshold: u128) -> Result<JsonValue, RpcError> {
    let accounts: Vec<String> = accounts.iter()
        .map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("sorting".into(), "true".into());
    arguments.insert("threshold".into(), threshold.to_string().into());
    arguments.insert("accounts".into(), accounts.as_slice().into());
    arguments.insert("count".into(), count.into());

    Ok(rpc.command("accounts_receivable".into(), arguments)
        .await?["blocks"].clone())
}

pub async fn accounts_representatives(rpc: &Rpc, accounts: &[Account]) -> Result<JsonValue, RpcError> {
    let accounts: Vec<String> = accounts.iter()
        .map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("accounts".into(), accounts.as_slice().into());

    Ok(rpc.command("accounts_representatives".into(), arguments)
        .await?["representatives"].clone())
}

pub async fn block_info(rpc: &Rpc, hash: [u8; 32]) -> Result<JsonValue, RpcError> {
    let mut arguments = Map::new();
    arguments.insert("hash".into(), hex::encode(hash).into());
    arguments.insert("json_block".into(), "true".into());

    rpc.command("block_info".into(), arguments).await
}

pub async fn blocks_info(rpc: &Rpc, hashes: &[[u8; 32]]) -> Result<JsonValue, RpcError> {
    let hashes: Vec<String> = hashes.iter()
        .map(hex::encode).collect();

    let mut arguments = Map::new();
    arguments.insert("hashes".into(), hashes.as_slice().into());
    arguments.insert("json_block".into(), "true".into());

    Ok(rpc.command("blocks_info".into(), arguments)
        .await?["blocks"].clone())
}

pub async fn process(rpc: &Rpc, block: Block) -> Result<JsonValue, RpcError> {
    if !block.block_type.is_state() {
        return Err(RpcError::LegacyBlockType)
    }

    let mut arguments = Map::new();
    arguments.insert("subtype".into(), block.block_type.to_string().into());
    arguments.insert("block".into(), JsonValue::Object(block_to_json(block)));
    arguments.insert("json_block".into(), "true".into());

    rpc.command("process".into(), arguments).await
}

pub async fn work_generate(rpc: &Rpc, work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> Result<JsonValue, RpcError> {
    let mut arguments = Map::new();
    arguments.insert("hash".into(), hex::encode(work_hash).into());
    arguments.insert("use_peers".into(), "true".into());
    if let Some(difficulty) = custom_difficulty {
        arguments.insert("difficulty".into(), hex::encode(difficulty).into());
    }

    rpc.command("work_generate".into(), arguments).await
}