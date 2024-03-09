# nanopyrs

Mid- and low-level access to functions and data types related to the Nano cryptocurrency.

This is, partially, a Rust rewrite of the Python [nanopy](https://github.com/npy0/nanopy) library. `nanopyrs` was initially part of another project, so some behaviors may seem odd.

There is not very much documentation at this time.

Things may or may not work before version `1.0.0`.

*Use at your own risk. I cannot guarantee that this library is perfect.*

## Feature Flags

### RPC

RPC functionality is enabled by the `rpc` feature, which is **disabled by default**.

Currently, only the following commands are officially supported: `account_balance`, `account_history`, `account_info`, `account_representative`, `accounts_balances`, `accounts_frontiers`, `accounts_receivable`, `accounts_representatives`, `block_info`, `blocks_info`, `process`, `work_generate`

. . . but any other command can be implemented manually with the help of the `command()` method of `nanopyrs::rpc::Rpc`, and various functions in `nanopyrs::rpc::util`.

### Camo Nano

Camo Nano functionality is enabled by the `camo` feature, which is **disabled by default**.

Note that Camo Nano is a **custom, experimental, and non-standard feature** of this library, and is generally not supported by wallets or the wider Nano ecosystem.

### Serde

[Serde](https://docs.rs/serde/latest/serde/) support is enabled by the `serde` feature, which is **disabled by default**.

## Shouldn't this be called 'nanors' since the 'py' in 'nanopy' means Python?

Maybe, but the name "nanors" was taken :(

## Licensing

This crate is open source and licensed under the MIT license. See the `LICENSE` file for more details.

## Credits

This library is heavily inspired by, and partially derived from, the [nanopy](https://github.com/npy0/nanopy) library, written by npy0.

The Base32 code was copied from the [feeless](https://github.com/feeless/feeless) library, written by gak