# nanopyrs

Mid- and low-level access to functions and data types related to the Nano cryptocurrency.

This is, partially, a Rust rewrite of the Python [nanopy](https://github.com/npy0/nanopy) library. `nanopyrs` was initially part of another project, so some behaviors may seem odd.

There is very little documentation at this time.

*Use at your own risk. I cannot guarantee that this library is perfect.*

## RPC

RPC functionality is enabled by the `rpc` feature, which is **enabled by default**.

Currently, only the following commands are officially supported: `account_balance`, `account_history`, `account_representative`, `accounts_balances`, `accounts_frontiers`, `accounts_receivable`, `accounts_representatives`, `block_info`, `blocks_info`, `process`, `work_generate`

. . . but any other command can be implemented manually with the help of the `command` method of `nanopyrs::rpc::Rpc`, and various functions in `nanopyrs::rpc::util`.

## Stealth Accounts

Stealth account functionality is enabled by the `stealth` feature, which is **disabled by default**.

Note that stealth accounts are a **custom, experimental, and non-standard feature** of this library, and are generally not supported by wallets or the wider Nano ecosystem.

## Shouldn't this be called 'nanors' since the 'py' in 'nanopy' means Python?

Maybe, but the name "nanors" was taken :(

## Licensing

This crate is open source and licensed under the MIT license. See the `LICENSE` file for more details.

## Credits

Heavily inspired by, and partially derived from, the [nanopy](https://github.com/npy0/nanopy) library, written by npy0.

The Base32 code was copied from the [feeless](https://github.com/feeless/feeless) library, written by gak