use super::util::{block_to_json, to_uppercase_hex};
use crate::{Account, Block};
use json::{Map, Value as JsonValue};
use serde_json as json;

pub fn account_balance(account: &Account) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "account_balance".into());
    arguments.insert("account".into(), account.into());
    JsonValue::Object(arguments)
}

pub fn account_history(
    account: &Account,
    count: usize,
    head: Option<[u8; 32]>,
    offset: Option<usize>,
) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "account_history".into());
    arguments.insert("raw".into(), true.into());
    arguments.insert("account".into(), account.into());
    arguments.insert("count".into(), count.to_string().into());
    if let Some(head) = head {
        arguments.insert("head".into(), hex::encode(head).into());
    }
    if let Some(offset) = offset {
        arguments.insert("offset".into(), offset.to_string().into());
    }
    JsonValue::Object(arguments)
}

pub fn account_info(account: &Account) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "account_info".into());
    arguments.insert("account".into(), account.into());
    arguments.insert("representative".into(), true.into());
    arguments.insert("weight".into(), true.into());
    arguments.insert("receivable".into(), true.into());
    JsonValue::Object(arguments)
}

pub fn accounts_balances(accounts: &[Account]) -> JsonValue {
    let accounts: Vec<String> = accounts.iter().map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("action".into(), "accounts_balances".into());
    arguments.insert("accounts".into(), accounts.as_slice().into());
    JsonValue::Object(arguments)
}

pub fn accounts_frontiers(accounts: &[Account]) -> JsonValue {
    let accounts: Vec<String> = accounts.iter().map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("action".into(), "accounts_frontiers".into());
    arguments.insert("accounts".into(), accounts.as_slice().into());
    JsonValue::Object(arguments)
}

pub fn accounts_receivable(accounts: &[Account], count: usize, threshold: u128) -> JsonValue {
    let accounts: Vec<String> = accounts.iter().map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("action".into(), "accounts_receivable".into());
    arguments.insert("sorting".into(), true.into());
    arguments.insert("threshold".into(), threshold.to_string().into());
    arguments.insert("accounts".into(), accounts.as_slice().into());
    arguments.insert("count".into(), count.to_string().into());
    JsonValue::Object(arguments)
}

pub fn accounts_representatives(accounts: &[Account]) -> JsonValue {
    let accounts: Vec<String> = accounts.iter().map(|account| account.to_string()).collect();

    let mut arguments = Map::new();
    arguments.insert("action".into(), "accounts_representatives".into());
    arguments.insert("accounts".into(), accounts.as_slice().into());
    JsonValue::Object(arguments)
}

pub fn block_info(hash: [u8; 32]) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "block_info".into());
    arguments.insert("hash".into(), to_uppercase_hex(&hash).into());
    arguments.insert("json_block".into(), true.into());
    JsonValue::Object(arguments)
}

pub fn blocks_info(hashes: &[[u8; 32]]) -> JsonValue {
    let hashes: Vec<String> = hashes.iter().map(|hash| to_uppercase_hex(hash)).collect();

    let mut arguments = Map::new();
    arguments.insert("action".into(), "blocks_info".into());
    arguments.insert("hashes".into(), hashes.as_slice().into());
    arguments.insert("json_block".into(), true.into());
    arguments.insert("include_not_found".into(), true.into());
    JsonValue::Object(arguments)
}

pub fn process(block: &Block) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "process".into());
    arguments.insert("subtype".into(), block.block_type.to_string().into());
    arguments.insert("block".into(), JsonValue::Object(block_to_json(block)));
    arguments.insert("json_block".into(), true.into());
    JsonValue::Object(arguments)
}

pub fn work_generate(work_hash: [u8; 32], custom_difficulty: Option<[u8; 8]>) -> JsonValue {
    let mut arguments = Map::new();
    arguments.insert("action".into(), "work_generate".into());
    arguments.insert("hash".into(), to_uppercase_hex(&work_hash).into());
    arguments.insert("use_peers".into(), true.into());
    if let Some(difficulty) = custom_difficulty {
        arguments.insert("difficulty".into(), hex::encode(difficulty).into());
    }
    JsonValue::Object(arguments)
}

#[cfg(test)]
mod tests {
    use crate::{Block, BlockType};
    use serde_json::json;

    #[test]
    fn account_balance() {
        let account = "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
            .try_into()
            .unwrap();
        let json = super::account_balance(&account);
        assert!(
            json == json!({
                "action": "account_balance",
                "account": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
            })
        )
    }

    #[test]
    fn account_history() {
        let account = "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est"
            .try_into()
            .unwrap();

        let json = super::account_history(&account, 3, None, Some(8));
        assert!(
            json == json!({
                "action": "account_history",
                "account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                "count": "3",
                "offset": "8",
                "raw": true
            })
        );

        let json = super::account_history(&account, 4, Some([255; 32]), None);
        assert!(
            json == json!({
                "action": "account_history",
                "account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                "head": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                "count": "4",
                "raw": true
            })
        )
    }

    #[test]
    fn account_info() {
        let account = "nano_1gyeqc6u5j3oaxbe5qy1hyz3q745a318kh8h9ocnpan7fuxnq85cxqboapu5"
            .try_into()
            .unwrap();

        let json = super::account_info(&account);
        assert!(
            json == json!({
                "action": "account_info",
                "account": "nano_1gyeqc6u5j3oaxbe5qy1hyz3q745a318kh8h9ocnpan7fuxnq85cxqboapu5",
                "representative": true,
                "weight": true,
                "receivable": true
            })
        );
    }

    #[test]
    fn accounts_balances() {
        let accounts = vec![
            "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                .try_into()
                .unwrap(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
                .try_into()
                .unwrap(),
        ];
        let json = super::accounts_balances(&accounts);
        assert!(
            json == json!({
                "action": "accounts_balances",
                "accounts": ["nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3", "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"]
            })
        )
    }

    #[test]
    fn accounts_frontiers() {
        let accounts = vec![
            "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                .try_into()
                .unwrap(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
                .try_into()
                .unwrap(),
        ];
        let json = super::accounts_frontiers(&accounts);
        assert!(
            json == json!({
                "action": "accounts_frontiers",
                "accounts": ["nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3", "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"]
            })
        )
    }

    #[test]
    fn accounts_receivable() {
        let accounts = vec![
            "nano_1111111111111111111111111111111111111111111111111117353trpda"
                .try_into()
                .unwrap(),
            "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                .try_into()
                .unwrap(),
        ];
        let json = super::accounts_receivable(&accounts, 9, 1000000000000000000000000);
        assert!(
            json == json!({
                "action": "accounts_receivable",
                "accounts": ["nano_1111111111111111111111111111111111111111111111111117353trpda", "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"],
                "count": "9",
                "threshold": "1000000000000000000000000",
                "sorting": true
            })
        )
    }

    #[test]
    fn accounts_representatives() {
        let accounts = vec![
            "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5"
                .try_into()
                .unwrap(),
            "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                .try_into()
                .unwrap(),
        ];
        let json = super::accounts_representatives(&accounts);
        assert!(
            json == json!({
                "action": "accounts_representatives",
                "accounts": ["nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5","nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"]
            })
        )
    }

    #[test]
    fn block_info() {
        let hash = hex::decode("87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9")
            .unwrap()
            .try_into()
            .unwrap();
        let json = super::block_info(hash);
        assert!(
            json == json!({
                "action": "block_info",
                "json_block": true,
                "hash": "87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9"
            })
        )
    }

    #[test]
    fn blocks_info() {
        let hashes =
            vec![
                hex::decode("87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9")
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ];
        let json = super::blocks_info(&hashes);
        assert!(
            json == json!({
                "action": "blocks_info",
                "json_block": true,
                "hashes": ["87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9"],
                "include_not_found": true
            })
        )
    }

    #[test]
    fn process() {
        let signature: [u8; 64] = hex::decode("A5DB164F6B81648F914E49CAB533900C389FAAD64FBB24F6902F9261312B29F730D07E9BCCD21D918301419B4E05B181637CF8419ED4DCBF8EF2539EB2467F07").unwrap().try_into().unwrap();
        let block = Block {
            block_type: BlockType::Send,
            account: "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z"
                .try_into()
                .unwrap(),
            previous: hex::decode(
                "6CDDA48608C7843A0AC1122BDD46D9E20E21190986B19EAC23E7F33F2E6A6766",
            )
            .unwrap()
            .try_into()
            .unwrap(),
            representative: "nano_3pczxuorp48td8645bs3m6c3xotxd3idskrenmi65rbrga5zmkemzhwkaznh"
                .try_into()
                .unwrap(),
            balance: 40200000001000000000000000000000000,
            link: hex::decode("87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9")
                .unwrap()
                .try_into()
                .unwrap(),
            signature: signature.try_into().unwrap(),
            work: hex::decode("000bc55b014e807d").unwrap().try_into().unwrap(),
        };
        let json = super::process(&block);
        assert!(
            json == json!({
                "action": "process",
                "json_block": true,
                "subtype": "send",
                "block": {
                    "type": "state",
                    "account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
                    "previous": "6CDDA48608C7843A0AC1122BDD46D9E20E21190986B19EAC23E7F33F2E6A6766",
                    "representative": "nano_3pczxuorp48td8645bs3m6c3xotxd3idskrenmi65rbrga5zmkemzhwkaznh",
                    "balance": "40200000001000000000000000000000000",
                    "link": "87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9",
                    "signature": "A5DB164F6B81648F914E49CAB533900C389FAAD64FBB24F6902F9261312B29F730D07E9BCCD21D918301419B4E05B181637CF8419ED4DCBF8EF2539EB2467F07",
                    "work": "000bc55b014e807d"
                }
            })
        )
    }

    #[test]
    fn work_generate() {
        let hash = hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2")
            .unwrap()
            .try_into()
            .unwrap();
        let json = super::work_generate(hash, None);
        assert!(
            json == json!({
                "action": "work_generate",
                "hash": "718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2",
                "use_peers": true
            })
        );

        let hash = hex::decode("718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2")
            .unwrap()
            .try_into()
            .unwrap();
        let json = super::work_generate(hash, Some([255; 8]));
        assert!(
            json == json!({
                "action": "work_generate",
                "hash": "718CC2121C3E641059BC1C2CFC45666C99E8AE922F7A807B7D07B62C995D79E2",
                "difficulty": "ffffffffffffffff",
                "use_peers": true
            })
        )
    }
}
