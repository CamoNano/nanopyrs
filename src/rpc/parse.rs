use super::{util::*, AccountInfo, Receivable, RpcError};
use crate::{block::check_work, Account, Block};
use hex::FromHexError;

pub fn account_balance(raw_json: JsonValue) -> Result<u128, RpcError> {
    let balances = u128_from_json(&raw_json["balance"])?;
    Ok(balances)
}

/// Will stop at first legacy block
pub fn account_history(raw_json: JsonValue, account: &Account) -> Result<Vec<Block>, RpcError> {
    let json_blocks = &raw_json["history"];
    let json_blocks = json_blocks
        .as_array()
        .ok_or(RpcError::InvalidJsonDataType)?;

    let mut blocks: Vec<Block> = vec![];
    for block in json_blocks {
        if trim_json(&block["type"].to_string()) != "state" {
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

        if !block.has_valid_signature() {
            return Err(RpcError::InvalidData);
        }

        blocks.push(block)
    }
    Ok(blocks)
}

pub fn account_info(raw_json: JsonValue) -> Result<AccountInfo, RpcError> {
    Ok(AccountInfo {
        frontier: bytes_from_json(&raw_json["frontier"])?,
        open_block: bytes_from_json(&raw_json["open_block"])?,
        representative_block: bytes_from_json(&raw_json["representative_block"])?,
        balance: u128_from_json(&raw_json["balance"])?,
        modified_timestamp: u64_from_json(&raw_json["modified_timestamp"])?,
        block_count: usize_from_json(&raw_json["block_count"])?,
        version: usize_from_json(&raw_json["account_version"])?,
        representative: account_from_json(&raw_json["representative"])?,
        weight: u128_from_json(&raw_json["weight"])?,
        receivable: usize_from_json(&raw_json["receivable"])?,
    })
}

pub fn account_representative(history: Vec<Block>) -> Result<Account, RpcError> {
    let last_block = history.first().ok_or(RpcError::InvalidJsonDataType)?;
    Ok(last_block.representative.clone())
}

pub fn accounts_balances(raw_json: JsonValue, accounts: &[Account]) -> Result<Vec<u128>, RpcError> {
    let mut balances = vec![];
    for account in accounts {
        balances.push(u128_from_json(
            &raw_json["balances"][account.to_string()]["balance"],
        )?)
    }
    Ok(balances)
}

pub fn accounts_frontiers(
    raw_json: JsonValue,
    accounts: &[Account],
) -> Result<Vec<Option<[u8; 32]>>, RpcError> {
    let mut frontiers = vec![];
    for account in accounts {
        let frontier = &raw_json["frontiers"][account.to_string()];
        if frontier.is_null() {
            frontiers.push(None);
            continue;
        }

        frontiers.push(Some(bytes_from_json(frontier)?))
    }
    Ok(frontiers)
}

pub fn accounts_receivable(
    raw_json: JsonValue,
    accounts: &[Account],
) -> Result<Vec<Vec<Receivable>>, RpcError> {
    let mut all_receivable = vec![];
    for account in accounts {
        let mut receivable = vec![];

        let account_hashes = map_keys_from_json(&raw_json["blocks"][&account.to_string()]);
        if account_hashes.is_err() {
            continue;
        }

        for hash in account_hashes? {
            let amount = u128_from_json(&raw_json["blocks"][&account.to_string()][&hash])?;
            let bytes = from_hex(hash)?
                .try_into()
                .map_err(|_| FromHexError::InvalidStringLength)?;

            receivable.push((account.clone(), bytes, amount).into());
        }
        all_receivable.push(receivable);
    }
    Ok(all_receivable)
}

pub fn accounts_representatives(
    raw_json: JsonValue,
    accounts: &[Account],
) -> Result<Vec<Option<Account>>, RpcError> {
    let mut representatives = vec![];
    for account in accounts {
        let representative = &raw_json["representatives"][account.to_string()];
        if representative.is_null() {
            representatives.push(None);
            continue;
        }
        let representative: Account = trim_json(&representative.to_string())
            .parse()
            .map_err(|_| RpcError::InvalidAccount)?;
        representatives.push(Some(representative));
    }
    Ok(representatives)
}

/// Legacy blocks will return `None`
pub fn block_info(raw_json: JsonValue) -> Result<Option<Block>, RpcError> {
    if trim_json(&raw_json["contents"]["type"].to_string()) != "state" {
        return Ok(None);
    }

    let block = block_from_info_json(&raw_json)?;
    if !block.has_valid_signature() {
        return Err(RpcError::InvalidData);
    }
    Ok(Some(block))
}

/// Legacy blocks will return `None`
pub fn blocks_info(
    raw_json: JsonValue,
    hashes: &[[u8; 32]],
) -> Result<Vec<Option<Block>>, RpcError> {
    let mut blocks = vec![];
    for hash in hashes {
        let block = &raw_json["blocks"][to_uppercase_hex(hash)];
        if block.is_null() {
            blocks.push(None);
            continue;
        }

        if trim_json(&block["contents"]["type"].to_string()) != "state" {
            blocks.push(None)
        }

        let block = block_from_info_json(block)?;
        if !block.has_valid_signature() {
            return Err(RpcError::InvalidData);
        }
        blocks.push(Some(block))
    }
    let _blocks: Vec<Block> = blocks.iter().flatten().cloned().collect();
    balances_sanity_check(&_blocks)?;
    Ok(blocks)
}

pub fn process(raw_json: JsonValue, hash: [u8; 32]) -> Result<[u8; 32], RpcError> {
    let rpc_hash: [u8; 32] = bytes_from_json(&raw_json["hash"])?;

    if rpc_hash != hash {
        return Err(RpcError::InvalidData);
    }
    Ok(hash)
}

pub fn work_generate(
    raw_json: JsonValue,
    work_hash: [u8; 32],
    custom_difficulty: Option<[u8; 8]>,
) -> Result<[u8; 8], RpcError> {
    let work: [u8; 8] = bytes_from_json(&raw_json["work"])?;

    let difficulty: [u8; 8] = if let Some(difficulty) = custom_difficulty {
        difficulty
    } else {
        bytes_from_json(&raw_json["difficulty"])?
    };

    match check_work(work_hash, difficulty, work) {
        true => Ok(work),
        false => Err(RpcError::InvalidData),
    }
}

#[cfg(test)]
mod tests {
    use super::to_uppercase_hex;
    use crate::{block::check_work, Account, Block, BlockType};
    use serde_json::json;

    #[test]
    fn account_balance() {
        let balance = super::account_balance(json!({
            "balance": "10000",
            "pending": "20000",
            "receivable": "30000"
        }))
        .unwrap();
        assert!(balance == 10000)
    }

    #[test]
    fn account_history() {
        let history = super::account_history(
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
        ).unwrap();

        let signature_1: [u8; 64] = hex::decode("3D45D616545D5CCE9766E3F6268C9AE88C0DCA61A6B034AE4804D46C9F75EA94BCA7E7AEBA46EA98117120FB491FE2F7D0664675EF36D8BFD9818DAE62209F06").unwrap().try_into().unwrap();
        let signature_2: [u8; 64] = hex::decode("5CB5A90D35301213B45706D1D5318D8E0B27DAA58782892411CB07F4E878E447F6B70AA7612B637FE7302D84750B621747303707ECE38C5F1F719D5446670207").unwrap().try_into().unwrap();
        assert!(
            history
                == vec!(
                    Block {
                        block_type: BlockType::Epoch,
                        account:
                            "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
                                .try_into()
                                .unwrap(),
                        previous: hex::decode(
                            "F8F83276ACCBFCCD13783309861EEE81E5FAF97BD28F84ED1DA62C7D4460E531"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        representative:
                            "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                                .try_into()
                                .unwrap(),
                        balance: 116024995745747584010554620134,
                        link: hex::decode(
                            "65706F636820763220626C6F636B000000000000000000000000000000000000"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        signature: signature_1.try_into().unwrap(),
                        work: hex::decode("894045458d590e7c").unwrap().try_into().unwrap()
                    },
                    Block {
                        block_type: BlockType::Send,
                        account:
                            "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
                                .try_into()
                                .unwrap(),
                        previous: hex::decode(
                            "EC9A8131D76E820818AD84554F3AE276542A642DB118C1B098C77A0A8A8446B5"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        representative:
                            "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                                .try_into()
                                .unwrap(),
                        balance: 116024995745747584010554620134,
                        link: hex::decode(
                            "C71CCE9A2BDD1DB6424B789885A8FBDA298E1BB009165B17209771182B0509C7"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        signature: signature_2.try_into().unwrap(),
                        work: hex::decode("b1bd2f559a745b5a").unwrap().try_into().unwrap()
                    }
                )
        )
    }

    #[test]
    fn account_info() {
        let info = super::account_info(json!({
            "frontier": "80A6745762493FA21A22718ABFA4F635656A707B48B3324198AC7F3938DE6D4F",
            "open_block": "0E3F07F7F2B8AEDEA4A984E29BFE1E3933BA473DD3E27C662EC041F6EA3917A0",
            "representative_block": "80A6745762493FA21A22718ABFA4F635656A707B48B3324198AC7F3938DE6D41",
            "balance": "11999999999999999918751838129509869131",
            "confirmed_balance": "11999999999999999918751838129509869131",
            "modified_timestamp": "1606934662",
            "block_count": "22966",
            "account_version": "1",
            "confirmed_height": "22966",
            "confirmed_frontier": "80A6745762493FA21A22718ABFA4F635656A707B48B3324198AC7F3938DE6D4F",
            "representative": "nano_1gyeqc6u5j3oaxbe5qy1hyz3q745a318kh8h9ocnpan7fuxnq85cxqboapu5",
            "confirmed_representative": "nano_1gyeqc6u5j3oaxbe5qy1hyz3q745a318kh8h9ocnpan7fuxnq85cxqboapu5",
            "weight": "11999999999999999918751838129509869132",
            "pending": "34",
            "receivable": "2",
            "confirmed_pending": "0",
            "confirmed_receivable": "2"
        })).unwrap();
        assert!(
            to_uppercase_hex(&info.frontier)
                == "80A6745762493FA21A22718ABFA4F635656A707B48B3324198AC7F3938DE6D4F"
        );
        assert!(
            to_uppercase_hex(&info.open_block)
                == "0E3F07F7F2B8AEDEA4A984E29BFE1E3933BA473DD3E27C662EC041F6EA3917A0"
        );
        assert!(
            to_uppercase_hex(&info.representative_block)
                == "80A6745762493FA21A22718ABFA4F635656A707B48B3324198AC7F3938DE6D41"
        );
        assert!(info.balance == 11999999999999999918751838129509869131);
        assert!(info.modified_timestamp == 1606934662);
        assert!(info.block_count == 22966);
        assert!(info.version == 1);
        assert!(
            info.representative
                == "nano_1gyeqc6u5j3oaxbe5qy1hyz3q745a318kh8h9ocnpan7fuxnq85cxqboapu5"
                    .parse()
                    .unwrap()
        );
        assert!(info.weight == 11999999999999999918751838129509869132);
        assert!(info.receivable == 2);
    }

    #[test]
    fn account_representative() {
        let signature: [u8; 64] = hex::decode("3D45D616545D5CCE9766E3F6268C9AE88C0DCA61A6B034AE4804D46C9F75EA94BCA7E7AEBA46EA98117120FB491FE2F7D0664675EF36D8BFD9818DAE62209F06").unwrap().try_into().unwrap();
        let representative = super::account_representative(vec![Block {
            block_type: BlockType::Send,
            account: "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
                .try_into()
                .unwrap(),
            previous: hex::decode(
                "EC9A8131D76E820818AD84554F3AE276542A642DB118C1B098C77A0A8A8446B5",
            )
            .unwrap()
            .try_into()
            .unwrap(),
            representative: "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                .try_into()
                .unwrap(),
            balance: 116024995745747584010554620134,
            link: hex::decode("C71CCE9A2BDD1DB6424B789885A8FBDA298E1BB009165B17209771182B0509C7")
                .unwrap()
                .try_into()
                .unwrap(),
            signature: signature.try_into().unwrap(),
            work: hex::decode("b1bd2f559a745b5a").unwrap().try_into().unwrap(),
        }])
        .unwrap();
        assert!(
            representative
                == "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                    .try_into()
                    .unwrap()
        )
    }

    #[test]
    fn accounts_balances() {
        let balances = super::accounts_balances(
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
            &vec![
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                    .try_into()
                    .unwrap(),
                "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
                    .try_into()
                    .unwrap(),
            ],
        )
        .unwrap();
        assert!(balances[0] == 325586539664609129644855132177);
        assert!(balances[1] == 10000000)
    }

    #[test]
    fn accounts_frontiers() {
        let frontiers = super::accounts_frontiers(
            json!({
                "frontiers":{
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": "791AF413173EEE674A6FCF633B5DFC0F3C33F397F0DA08E987D9E0741D40D81A",
                    "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7": "6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F"
                },
                "errors":{
                    "nano_1hrts7hcoozxccnffoq9hqhngnn9jz783usapejm57ejtqcyz9dpso1bibuy": "Account not found"
                }
            }),
            &vec!(
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".try_into().unwrap(),
                "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7".try_into().unwrap(),
                "nano_1hrts7hcoozxccnffoq9hqhngnn9jz783usapejm57ejtqcyz9dpso1bibuy".try_into().unwrap()
            )
        ).unwrap();

        let hash_1: [u8; 32] =
            hex::decode("791AF413173EEE674A6FCF633B5DFC0F3C33F397F0DA08E987D9E0741D40D81A")
                .unwrap()
                .try_into()
                .unwrap();
        let hash_2: [u8; 32] =
            hex::decode("6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F")
                .unwrap()
                .try_into()
                .unwrap();
        assert!(frontiers[0] == Some(hash_1));
        assert!(frontiers[1] == Some(hash_2));
        assert!(frontiers[2].is_none())
    }

    #[test]
    fn accounts_receivable() {
        let receivable = super::accounts_receivable(
            json!({
                "blocks":{
                    "nano_1111111111111111111111111111111111111111111111111117353trpda": {
                        "142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D": "6000000000000000000000000000000",
                        "6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F": "9000000000000000000000000000005"
                    },
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": {
                        "4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74": "106370018000000000000000000000000"
                    }
                }
            }),
            &vec!(
                "nano_1111111111111111111111111111111111111111111111111117353trpda".try_into().unwrap(),
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".try_into().unwrap()
            )
        ).unwrap();

        let hash_1: [u8; 32] =
            hex::decode("142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D")
                .unwrap()
                .try_into()
                .unwrap();
        let hash_2: [u8; 32] =
            hex::decode("6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F")
                .unwrap()
                .try_into()
                .unwrap();
        let hash_3: [u8; 32] =
            hex::decode("4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74")
                .unwrap()
                .try_into()
                .unwrap();

        assert!(
            receivable[0][0].recipient
                == "nano_1111111111111111111111111111111111111111111111111117353trpda"
                    .parse()
                    .unwrap()
        );
        assert!(receivable[0][0].block_hash == hash_1);
        assert!(receivable[0][0].amount == 6000000000000000000000000000000);

        assert!(
            receivable[0][1].recipient
                == "nano_1111111111111111111111111111111111111111111111111117353trpda"
                    .parse()
                    .unwrap()
        );
        assert!(receivable[0][1].block_hash == hash_2);
        assert!(receivable[0][1].amount == 9000000000000000000000000000005);

        assert!(
            receivable[1][0].recipient
                == "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                    .parse()
                    .unwrap()
        );
        assert!(receivable[1][0].block_hash == hash_3);
        assert!(receivable[1][0].amount == 106370018000000000000000000000000);
    }

    #[test]
    fn accounts_representatives() {
        let representatives = super::accounts_representatives(
            json!({
                "representatives":{
                    "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5": "nano_3hd4ezdgsp15iemx7h81in7xz5tpxi43b6b41zn3qmwiuypankocw3awes5k",
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                },
                "errors":{
                    "nano_1hrts7hcoozxccnffoq9hqhngnn9jz783usapejm57ejtqcyz9dpso1bibuy": "Account not found"
                }
            }),
            &vec!(
                "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5".try_into().unwrap(),
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".try_into().unwrap(),
                "nano_1hrts7hcoozxccnffoq9hqhngnn9jz783usapejm57ejtqcyz9dpso1bibuy".try_into().unwrap()
            )
        ).unwrap();

        assert!(
            representatives[0]
                == Some(
                    "nano_3hd4ezdgsp15iemx7h81in7xz5tpxi43b6b41zn3qmwiuypankocw3awes5k"
                        .try_into()
                        .unwrap()
                )
        );
        assert!(
            representatives[1]
                == Some(
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                        .try_into()
                        .unwrap()
                )
        );
        assert!(representatives[2].is_none())
    }

    #[test]
    fn block_info() {
        // block found
        let signature: [u8; 64] = hex::decode("82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501").unwrap().try_into().unwrap();
        let block = super::block_info(
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
        ).unwrap();

        assert!(
            block
                == Some(Block {
                    block_type: BlockType::Send,
                    account: "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
                        .try_into()
                        .unwrap(),
                    previous: hex::decode(
                        "CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                    representative:
                        "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                            .try_into()
                            .unwrap(),
                    balance: 5606157000000000000000000000000000000,
                    link: hex::decode(
                        "5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                    signature: signature.try_into().unwrap(),
                    work: hex::decode("8a142e07a10996d5").unwrap().try_into().unwrap()
                })
        );

        // block not found
        let block = super::block_info(json!({"error":"Block not found"})).unwrap();
        assert!(block.is_none())
    }

    #[test]
    fn blocks_info() {
        let blocks = super::blocks_info(
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
            &[
                hex::decode("87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9").unwrap().try_into().unwrap(),
                hex::decode("5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5").unwrap().try_into().unwrap()
            ]
        ).unwrap();

        let signature: [u8; 64] = hex::decode("82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501").unwrap().try_into().unwrap();
        assert!(
            blocks
                == vec!(
                    Some(Block {
                        block_type: BlockType::Send,
                        account:
                            "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
                                .try_into()
                                .unwrap(),
                        previous: hex::decode(
                            "CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        representative:
                            "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou"
                                .try_into()
                                .unwrap(),
                        balance: 5606157000000000000000000000000000000,
                        link: hex::decode(
                            "5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap(),
                        signature: signature.try_into().unwrap(),
                        work: hex::decode("8a142e07a10996d5").unwrap().try_into().unwrap()
                    }),
                    None
                )
        )
    }

    #[test]
    fn process() {
        let block_hash: [u8; 32] =
            hex::decode("E2FB233EF4554077A7BF1AA85851D5BF0B36965D2B0FB504B2BC778AB89917D3")
                .unwrap()
                .try_into()
                .unwrap();
        let hash = super::process(
            json!({
                "hash": "E2FB233EF4554077A7BF1AA85851D5BF0B36965D2B0FB504B2BC778AB89917D3"
            }),
            hex::decode("E2FB233EF4554077A7BF1AA85851D5BF0B36965D2B0FB504B2BC778AB89917D3")
                .unwrap()
                .try_into()
                .unwrap(),
        )
        .unwrap();
        assert!(hash == block_hash)
    }

    #[test]
    fn work_generate() {
        // valid
        let work = super::work_generate(
            json!({
                "work": "2b3d689bbcb21dca",
                "difficulty": "fffffff93c41ec94", // of the resulting work
                "multiplier": "1.182623871097636", // since v19.0, calculated from default base difficulty
                "hash": "718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2" // since v20.0
            }),
            hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2")
                .unwrap()
                .try_into()
                .unwrap(),
            None,
        )
        .unwrap();

        assert!(check_work(
            hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("fffffff93c41ec94").unwrap().try_into().unwrap(),
            work
        ));

        // invalid
        super::work_generate(
            json!({
                "work": "2b3d689bbcb21d00",
                "difficulty": "fffffff93c41ec94", // of the resulting work
                "multiplier": "1.182623871097636", // since v19.0, calculated from default base difficulty
                "hash": "718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2" // since v20.0
            }),
            hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2")
                .unwrap()
                .try_into()
                .unwrap(),
            None,
        )
        .unwrap_err();
    }
}
