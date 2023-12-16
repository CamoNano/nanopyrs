use crate::core::{Account, Block, BlockType, block::check_work};
use super::internal::{InternalRpc, trim_json};
use super::error::RpcError;

use serde_json as json;
use json::{
    Value as JsonValue,
    Map
};

pub mod util {
    use super::*;

    pub use json::{
        Value as JsonValue,
        Map
    };

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

    /// specific to account_history
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

    /// specific to block_info and blocks_info
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
        json_block.insert("previous".into(), hex::encode(block.previous).into());
        json_block.insert("representative".into(), block.representative.into());
        json_block.insert("balance".into(), block.balance.to_string().into());
        json_block.insert("link".into(), hex::encode(block.link).into());
        json_block.insert("signature".into(), hex::encode(block.signature.to_bytes()).into());
        json_block.insert("work".into(), hex::encode(block.work).into());
        json_block
    }

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
}
use util::*;

/// See Nano's [RPC documentation](https://docs.nano.org/commands/rpc-protocol/) for details
#[derive(Debug, Clone)]
pub struct Rpc {
    internal: InternalRpc
}
impl Rpc {
    pub fn new(url: &str) -> Rpc {
        Rpc { internal: InternalRpc::new(url) }
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
        let balances = u128_from_json(&raw_json["balance"])?;
        Ok(balances)
    }

    pub async fn account_history(&self, account: &Account, count: usize, head: Option<[u8; 32]>) -> Result<Vec<Block>, RpcError> {
        let mut arguments = Map::new();
        arguments.insert("raw".into(), "true".into());
        if let Some(head) = head {
            arguments.insert("head".into(), hex::encode(head).into());
        }

        let raw_json = self.internal
            .account_history(account.to_string(), count, arguments).await?;
        let json_blocks = raw_json["history"].clone();
        let json_blocks = RpcError::from_option(json_blocks.as_array())?;

        let mut blocks: Vec<Block> = vec!();
        for block in json_blocks {
            let block = block_from_history_json(block)?;

            if &block.account != account {
                return Err(RpcError::InvalidData);
            }

            if let Some(successor_block) = blocks.last() {
                if successor_block.previous != block.hash() {
                    return Err(RpcError::InvalidData);
                }
            }

            blocks.push(block)
        }

        if let Some(newest_block) = blocks.get(0) {
            if !newest_block.has_valid_signature() {
                return Err(RpcError::InvalidData);
            }
        }
        Ok(blocks)
    }

    /// Indirect, relies on account_history
    pub async fn account_representative(&self, account: &Account) -> Result<Account, RpcError> {
        let history = self.account_history(account, 1, None).await?;
        let last_block = RpcError::from_option(history.get(0))?;
        Ok(last_block.representative.clone())
    }

    pub async fn accounts_balances(&self, accounts: &[Account]) -> Result<Vec<u128>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();
        let raw_json = self.internal.accounts_balances(&accounts, Map::new()).await?;

        let mut balances = vec!();
        for account in &accounts {
            balances.push(u128_from_json(&raw_json[account]["balance"])?)
        }
        Ok(balances)
    }

    pub async fn accounts_frontiers(&self, accounts: &[Account]) -> Result<Vec<[u8; 32]>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();

        let raw_json = self.internal.accounts_frontiers(&accounts)
            .await?["frontiers"].clone();

        let mut frontiers = vec!();
        for account in &accounts {
            let frontier = raw_json[account].clone();
            if frontier.is_null() {
                frontiers.push([0; 32]);
                continue;
            }

            let frontier = hex::decode(
                trim_json(frontier.to_string())
            )?;
            let frontier = frontier.try_into().or(Err(
                RpcError::ParseError("failed to parse hashes".into())
            ))?;

            frontiers.push(frontier)
        }
        Ok(frontiers)
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

        let mut all_hashes = vec!();
        for account in &accounts {
            let mut hashes = vec!();

            let account_hashes = map_keys_from_json(raw_json[&account].clone());
            if account_hashes.is_err() {
                continue;
            }

            for hash in account_hashes? {
                let amount = u128_from_json(&raw_json[&account][&hash])?;
                let bytes = hex::decode(trim_json(hash))?;
                let bytes = bytes.try_into().or(Err(
                    RpcError::ParseError("failed to parse hashes".into())
                ))?;

                hashes.push((bytes, amount));
            }
            all_hashes.push(hashes);
        }
        Ok(all_hashes)
    }

    pub async fn accounts_representatives(&self, accounts: &[Account]) -> Result<Vec<Option<Account>>, RpcError> {
        if accounts.is_empty() {
            return Ok(vec!());
        }

        let accounts: Vec<String> = accounts.iter()
            .map(|account| account.to_string()).collect();

        let raw_json = self.internal.accounts_representatives(&accounts).await?["representatives"].clone();
        let mut representatives = vec!();
        for account in accounts {
            let representative = raw_json[account].clone();
            if representative.is_null() {
                representatives.push(None)
            }

            representatives.push(
                Some(Account::try_from(&representative.to_string())?)
            );
        }
        Ok(representatives)
    }

    pub async fn block_info(&self, hash: [u8; 32]) -> Result<Block, RpcError> {
        let raw_json = self.internal.block_info(hex::encode(hash)).await?;
        let block = block_from_info_json(&raw_json)?;
        if !block.has_valid_signature() {
            return Err(RpcError::InvalidData)
        }
        Ok(block)
    }

    pub async fn blocks_info(&self, hashes: &[[u8; 32]]) -> Result<Vec<Block>, RpcError> {
        if hashes.is_empty() {
            return Ok(vec!());
        }

        let hashes: Vec<String> = hashes.iter()
            .map(hex::encode).collect();

        let raw_json = self.internal.blocks_info(&hashes).await?["blocks"].clone();
        let mut blocks = vec!();
        for hash in hashes {
            let block = block_from_info_json(&raw_json[hash])?;
            if !block.has_valid_signature() {
                return Err(RpcError::InvalidData);
            }
            blocks.push(block)
        }
        balances_sanity_check(&blocks)?;
        Ok(blocks)
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
        let rpc_hash = hex::decode(trim_json(raw_json["hash"].to_string()))?;
        let rpc_hash: [u8; 32] = rpc_hash.try_into().or(Err(
            RpcError::ParseError("failed to process block".into())
        ))?;

        if rpc_hash != hash {
            return Err(RpcError::InvalidData)
        }
        Ok(hash)
    }

    pub async fn work_generate(&self, work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> Result<[u8; 8], RpcError> {
        let mut map = Map::new();
        if let Some(difficulty) = custom_difficulty {
            map.insert("difficulty".into(), hex::encode(difficulty).into());
        }

        let raw_json = self.internal.work_generate(hex::encode(work_hash), map).await?;

        let work = hex::decode(trim_json(raw_json["work"].to_string()))?;
        let work: [u8; 8] = work.try_into().or(Err(
            RpcError::ParseError("failed to generate work".into())
        ))?;
        let difficulty = hex::decode(trim_json(raw_json["difficulty"].to_string()))?;
        let difficulty: [u8; 8] = difficulty.try_into().or(Err(
            RpcError::ParseError("failed to verify work".into())
        ))?;

        match check_work(work_hash, difficulty, work) {
            true => Ok(work),
            false => Err(RpcError::InvalidData)
        }
    }
}