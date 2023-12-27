use super::{RpcError, util::*};
use crate::{Account, Block, block::check_work};

pub async fn account_balance(raw_json: JsonValue) -> Result<u128, RpcError> {
    let balances = u128_from_json(&raw_json["balance"])?;
    Ok(balances)
}

/// Will stop at first legacy block
pub async fn account_history(raw_json: JsonValue, account: &Account) -> Result<Vec<Block>, RpcError> {
    let json_blocks = raw_json["history"].clone();
    let json_blocks = RpcError::from_option(json_blocks.as_array())?;

    let mut blocks: Vec<Block> = vec!();
    for block in json_blocks {
        if trim_json(block["type"].to_string()) != "state" {
            break;
        }

        let mut block = block_from_history_json(block)?;
        // "account" field may be wrong due to a compatibility feature in the RPC protocol
        block.account = account.clone();

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
    } else if !json_blocks.is_empty() {
        return Err(RpcError::InvalidData);
    }
    Ok(blocks)
}

pub async fn account_representative(history: Vec<Block>) -> Result<Account, RpcError> {
    let last_block = RpcError::from_option(history.get(0))?;
    Ok(last_block.representative.clone())
}

pub async fn accounts_balances(raw_json: JsonValue, accounts: Vec<String>) -> Result<Vec<u128>, RpcError> {
    let mut balances = vec!();
    for account in &accounts {
        balances.push(u128_from_json(&raw_json["balances"][account]["balance"])?)
    }
    Ok(balances)
}

pub async fn accounts_frontiers(raw_json: JsonValue, accounts: Vec<String>) -> Result<Vec<[u8; 32]>, RpcError> {
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

pub async fn accounts_receivable(raw_json: JsonValue, accounts: Vec<String>) -> Result<Vec<Vec<([u8; 32], u128)>>, RpcError> {
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

pub async fn accounts_representatives(raw_json: JsonValue, accounts: Vec<String>) -> Result<Vec<Option<Account>>, RpcError> {
    let mut representatives = vec!();
    for account in accounts {
        let representative = raw_json["representatives"][account].clone();
        if representative.is_null() {
            representatives.push(None)
        }
        representatives.push(
            Some(Account::try_from(trim_json(representative.to_string()))?)
        );
    }
    Ok(representatives)
}

/// Legacy blocks will return `None`
pub async fn block_info(raw_json: JsonValue) -> Result<Option<Block>, RpcError> {
    if trim_json(raw_json["type"].to_string()) != "state" {
        return Ok(None)
    }

    let block = block_from_info_json(&raw_json)?;
    if !block.has_valid_signature() {
        return Err(RpcError::InvalidData)
    }
    Ok(Some(block))
}

/// Legacy blocks will return `None`
pub async fn blocks_info(raw_json: JsonValue, hashes: Vec<String>) -> Result<Vec<Option<Block>>, RpcError> {
    let mut blocks = vec!();
    for hash in hashes {
        if trim_json(raw_json["blocks"][&hash]["type"].to_string()) != "state" {
            blocks.push(None)
        }

        let block = block_from_info_json(&raw_json["blocks"][hash])?;
        if !block.has_valid_signature() {
            return Err(RpcError::InvalidData);
        }
        blocks.push(Some(block))
    }
    let _blocks: Vec<Block> = blocks
        .iter().flatten().cloned().collect();
    balances_sanity_check(&_blocks)?;
    Ok(blocks)
}

pub async fn process(raw_json: JsonValue, hash: [u8; 32]) -> Result<[u8; 32], RpcError> {
    let rpc_hash = hex::decode(trim_json(raw_json["hash"].to_string()))?;
    let rpc_hash: [u8; 32] = rpc_hash.try_into().or(Err(
        RpcError::ParseError("failed to process block".into())
    ))?;

    if rpc_hash != hash {
        return Err(RpcError::InvalidData)
    }
    Ok(hash)
}

pub async fn work_generate(raw_json: JsonValue, work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> Result<[u8; 8], RpcError> {
    let work = hex::decode(trim_json(raw_json["work"].to_string()))?;
    let work: [u8; 8] = work.try_into().or(Err(
        RpcError::ParseError("failed to generate work".into())
    ))?;

    let difficulty: [u8; 8] = if let Some(difficulty) = custom_difficulty {
        difficulty
    } else {
        hex::decode(trim_json(raw_json["difficulty"].to_string()))?
            .try_into()
            .or(Err(RpcError::ParseError("failed to verify work".into())))?
    };

    match check_work(work_hash, difficulty, work) {
        true => Ok(work),
        false => Err(RpcError::InvalidData)
    }
}

#[cfg(test)]
mod tests {
    use crate::Account;
    use serde_json::json;
    use tokio::runtime::Runtime;

    fn runtime() -> Runtime {
        tokio::runtime::Builder::new_current_thread().build().unwrap()
    }

    #[test]
    fn account_balance() {
        runtime().block_on(
            super::account_balance(
                json!({
                    "balance": "10000",
                    "pending": "20000",
                    "receivable": "30000"
                })
            )
        ).unwrap();
    }

    #[test]
    fn account_history() {
        runtime().block_on(
            super::account_history(
                json!({
                    "account":"nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                    "history":[
                        {
                            "type":"state",
                            "representative":"nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou",
                            "link":"65706F636820763220626C6F636B000000000000000000000000000000000000",
                            "balance":"116024995745747584010554620134",
                            "previous":"F8F83276ACCBFCCD13783309861EEE81E5FAF97BD28F84ED1DA62C7D4460E531",
                            "subtype":"epoch",
                            "account":"nano_3qb6o6i1tkzr6jwr5s7eehfxwg9x6eemitdinbpi7u8bjjwsgqfj4wzser3x",
                            "local_timestamp":"1598397125",
                            "height":"281",
                            "hash":"BFD5D5214A93E614D64A7C05624F69E6CFD4F1CED3C5926562F282DF135B15CF",
                            "confirmed":"true",
                            "work":"894045458d590e7c",
                            "signature":"3D45D616545D5CCE9766E3F6268C9AE88C0DCA61A6B034AE4804D46C9F75EA94BCA7E7AEBA46EA98117120FB491FE2F7D0664675EF36D8BFD9818DAE62209F06",
                            "amount_nano":"Error: First parameter, raw amount is missing."
                        },
                        {
                            "type":"state",
                            "representative":"nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou",
                            "link":"C71CCE9A2BDD1DB6424B789885A8FBDA298E1BB009165B17209771182B0509C7",
                            "balance":"116024995745747584010554620134",
                            "previous":"EC9A8131D76E820818AD84554F3AE276542A642DB118C1B098C77A0A8A8446B5",
                            "subtype":"send",
                            "account":"nano_3jrwstf4qqaxps36py6ripnhqpjbjrfu14apdedk37uj51oic4g94qcabf1i",
                            "amount":"22066000000000000000000000000000000",
                            "local_timestamp":"1575915652",
                            "height":"280",
                            "hash":"F8F83276ACCBFCCD13783309861EEE81E5FAF97BD28F84ED1DA62C7D4460E531",
                            "confirmed":"true",
                            "work":"b1bd2f559a745b5a",
                            "signature":"5CB5A90D35301213B45706D1D5318D8E0B27DAA58782892411CB07F4E878E447F6B70AA7612B637FE7302D84750B621747303707ECE38C5F1F719D5446670207",
                            "amount_nano":"22066"
                        }
                    ],
                    "previous":"EC9A8131D76E820818AD84554F3AE276542A642DB118C1B098C77A0A8A8446B5"
                }),
                &Account::try_from("nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est").unwrap()
            )
        ).unwrap();
    }

    #[test]
    fn accounts_balances() {
        runtime().block_on(
            super::accounts_balances(
                json!({
                    "balances":{
                        "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3":{
                            "balance": "325586539664609129644855132177",
                            "pending": "2309372032769300000000000000000000",
                            "receivable": "2309372032769300000000000000000000"
                        },
                        "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7":{
                            "balance": "10000000",
                            "pending": "0",
                            "receivable": "0"
                        }
                    }
                }),
                vec!(
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".into(),
                    "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7".into()
                )
            )
        ).unwrap();
    }

    #[test]
    fn accounts_frontiers() {
        runtime().block_on(
            super::accounts_frontiers(
                json!({
                    "frontiers":{
                        "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": "791AF413173EEE674A6FCF633B5DFC0F3C33F397F0DA08E987D9E0741D40D81A",
                        "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7": "6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F"
                    }
                }),
                vec!(
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".into(),
                    "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7".into()
                )
            )
        ).unwrap();
    }

    #[test]
    fn accounts_receivable() {
        runtime().block_on(
            super::accounts_receivable(
                json!({
                    "blocks":{
                        "nano_1111111111111111111111111111111111111111111111111117353trpda": ["142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D"],
                        "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": ["4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74"]
                    }
                }),
                vec!(
                    "nano_1111111111111111111111111111111111111111111111111117353trpda".into(),
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".into()
                )
            )
        ).unwrap();
    }

    #[test]
    fn accounts_representatives() {
        runtime().block_on(
            super::accounts_representatives(
                json!({
                    "representatives":{
                        "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5": "nano_3hd4ezdgsp15iemx7h81in7xz5tpxi43b6b41zn3qmwiuypankocw3awes5k",
                        "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                    }
                }),
                vec!(
                    "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5".into(),
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".into()
                )
            )
        ).unwrap();
    }

    #[test]
    fn block_info() {
        runtime().block_on(
            super::block_info(
                json!({
                    "block_account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                    "amount": "30000000000000000000000000000000000",
                    "balance": "5606157000000000000000000000000000000",
                    "height": "58",
                    "local_timestamp": "0",
                    "successor": "8D3AB98B301224253750D448B4BD997132400CEDD0A8432F775724F2D9821C72",
                    "confirmed": "true",
                    "contents":{
                        "type": "state",
                        "account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                        "previous": "CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E",
                        "representative": "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou",
                        "balance": "5606157000000000000000000000000000000",
                        "link": "5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5",
                        "link_as_account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
                        "signature": "82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501",
                        "work": "8a142e07a10996d5"
                    },
                    "subtype": "send"
                })
            )
        ).unwrap();
    }

    #[test]
    fn blocks_info() {
        runtime().block_on(
            super::blocks_info(
                json!({
                    "blocks": {
                        "87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9": {
                            "block_account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                            "amount": "30000000000000000000000000000000000",
                            "balance": "5606157000000000000000000000000000000",
                            "height": "58",
                            "local_timestamp": "0",
                            "successor": "8D3AB98B301224253750D448B4BD997132400CEDD0A8432F775724F2D9821C72",
                            "confirmed": "true",
                            "contents": {
                                "type": "state",
                                "account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                                "previous": "CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E",
                                "representative": "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou",
                                "balance": "5606157000000000000000000000000000000",
                                "link": "5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5",
                                "link_as_account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
                                "signature": "82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501",
                                "work": "8a142e07a10996d5"
                            },
                            "subtype": "send"
                        }
                    }
                }),
                vec!("87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9".into())
            )
        ).unwrap();
    }

    #[test]
    fn process() {
        runtime().block_on(
            super::process(
                json!({
                    "hash": "E2FB233EF4554077A7BF1AA85851D5BF0B36965D2B0FB504B2BC778AB89917D3"
                }),
                hex::decode("E2FB233EF4554077A7BF1AA85851D5BF0B36965D2B0FB504B2BC778AB89917D3").unwrap().try_into().unwrap()
            )
        ).unwrap();
    }

    #[test]
    fn work_generate() {
        runtime().block_on(
            super::work_generate(
                json!({
                    "work": "2b3d689bbcb21dca",
                    "difficulty": "fffffff93c41ec94", // of the resulting work
                    "multiplier": "1.182623871097636", // since v19.0, calculated from default base difficulty
                    "hash": "718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2" // since v20.0
                }),
                hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2").unwrap().try_into().unwrap(),
                None
            )
        ).unwrap();
    }
}