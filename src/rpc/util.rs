use super::RpcError;
use crate::{Account, Block, BlockType};

pub use serde_json::{
    Value as JsonValue,
    Map
};

pub fn trim_json(value: String) -> String {
    value.trim_matches('\"').into()
}

pub fn to_uppercase_hex(bytes: &[u8]) -> String {
    hex::encode(bytes).to_uppercase()
}

/// Get the keys in a Json map.
pub fn map_keys_from_json(value: JsonValue) -> Result<Vec<String>, RpcError> {
    let keys: Vec<String> = RpcError::from_option(
        value.as_object()
    )?.keys().cloned().collect();
    Ok(keys)
}

pub fn u128_from_json(value: &JsonValue) -> Result<u128, RpcError> {
    Ok(
        trim_json(value.to_string())
        .parse::<u128>()?
    )
}

pub fn bytes_from_json<const T: usize>(value: &JsonValue) -> Result<[u8; T], RpcError> {
    hex::decode(trim_json(value.to_string()))?
        .try_into()
        .or(Err(RpcError::ParseError("failed to parse hex".into())))
}

pub fn account_from_json(value: &JsonValue) -> Result<Account, RpcError> {
    Ok(Account::try_from(
        &trim_json(value.to_string())
    )?)
}

pub fn block_from_json(block: &JsonValue, block_type: BlockType) -> Result<Block, RpcError> {
    Ok(Block{
        block_type,
        account: account_from_json(&block["account"])?,
        previous: bytes_from_json(&block["previous"])?,
        representative: account_from_json(&block["representative"])?,
        balance: u128_from_json(&block["balance"])?,
        link: bytes_from_json(&block["link"])?,
        signature: bytes_from_json::<64>(&block["signature"])?.try_into().unwrap(),
        work: bytes_from_json(&block["work"])?
    })
}

/// Specific to `account_history`
pub(crate) fn block_from_history_json(block: &JsonValue) -> Result<Block, RpcError> {
    let block_type = trim_json(block["type"].to_string());
    let block_type = if &block_type == "state" {
        // state blocks
        BlockType::from_subtype_string(
            &trim_json(block["subtype"].to_string())
        )
    } else {
        // legacy blocks (shouldn't be needed)
        Some(BlockType::Legacy(block_type))
    };

    block_from_json(block, RpcError::from_option(block_type)?)
}

/// Specific to `block_info` and `blocks_info`
pub(crate) fn block_from_info_json(block: &JsonValue) -> Result<Block, RpcError> {
    let contents = block["contents"].clone();
    let block_type = trim_json(contents["type"].to_string());
    let block_type = if &block_type == "state" {
        // state blocks
        BlockType::from_subtype_string(
            &trim_json(block["subtype"].to_string())
        )
    } else {
        // legacy blocks (shouldn't be needed)
        Some(BlockType::Legacy(block_type))
    };

    block_from_json(&contents, RpcError::from_option(block_type)?)
}

/// **Does not handle "subtype" field**
pub fn block_to_json(block: Block) -> Map<String, JsonValue> {
    let block_type = match block.block_type {
        BlockType::Legacy(block_type) => block_type,
        _ => "state".into()
    };

    let mut json_block = Map::new();
    json_block.insert("type".into(), block_type.into());
    json_block.insert("account".into(), block.account.into());
    json_block.insert("previous".into(), to_uppercase_hex(&block.previous).into());
    json_block.insert("representative".into(), block.representative.into());
    json_block.insert("balance".into(), block.balance.to_string().into());
    json_block.insert("link".into(), to_uppercase_hex(&block.link).into());
    json_block.insert("signature".into(), to_uppercase_hex(&block.signature.to_bytes()).into());
    json_block.insert("work".into(), hex::encode(block.work).into());
    json_block
}

/// Sanity check to ensure that no overflow occurs
pub fn balances_sanity_check(blocks: &[Block]) -> Result<(), RpcError> {
    let mut total: u128 = 0;
    let mut overflow: bool;
    for block in blocks {
        (total, overflow) = total.overflowing_add(block.balance);
        if overflow {
            return Err(RpcError::InvalidData)
        }
    }
    Ok(())
}