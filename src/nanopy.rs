// Copyright (c) 2023 npy0

// This is (mostly) a rust re-write of nanopy, a Python library for Nano by npy0.
// Shouldn't this be called 'nanors' since the 'py' in 'nanopy' means Python? Nanors was taken, oh well.

// https://docs.nano.org/protocol-design/

use crate::scalar;
use super::error::NanoError;
use super::hashes::*;
use super::{SecretBytes, Scalar, Account, Key, Signature, Block, base32, try_compressed_from_slice};
use curve25519_dalek::{
    edwards::CompressedEdwardsY,
    constants::ED25519_BASEPOINT_POINT as G,
};

pub(crate) fn account_encode(key: &CompressedEdwardsY) -> String {
    let key = key.as_bytes();

    let mut checksum = blake2b_checksum(key);
    checksum.reverse();

    let mut account = "nano_".to_string();
    let data = [[0, 0, 0].as_slice(), key, &checksum].concat();
    account.push_str(&base32::encode(&data)[4..]);
    account
}

pub(crate) fn account_decode(account: &str) -> Result<CompressedEdwardsY, NanoError> {
    if account.len() != 65 {
        return Err(NanoError::InvalidLength)
    }

    if &account[..5] != "nano_" {
        return Err(NanoError::InvalidFormatting)
    }

    let mut data = "1111".to_string();
    data.push_str(&account[5..]);

    let data = base32::decode(&data)
        .ok_or(NanoError::InvalidBase32)?;

    let checksum = &data[35..40];
    let key = &data[3..35];
    let mut calculated_checksum = blake2b_checksum(key);
    calculated_checksum.reverse();

    if checksum != calculated_checksum {
        return Err(NanoError::InvalidChecksum)
    }
    try_compressed_from_slice(key)
}

/// Return the "sub"-seed for the seed's account
pub fn get_account_seed(master_seed: &SecretBytes<32>, i: u32) -> SecretBytes<32> {
    blake2b256(&[master_seed.as_slice(), &i.to_be_bytes()].concat())
}

/// Return the private key, in `Scalar` form, for the seed's account
pub fn get_account_scalar(master_seed: &SecretBytes<32>, i: u32) -> Scalar {
    blake2b_scalar(get_account_seed(master_seed, i).as_ref())
}

/// Get work using the local CPU (likely very slow)
pub fn get_local_work(block_hash: [u8; 32], difficulty: [u8; 8]) -> [u8; 8] {
    let mut data: [u8; 40] = [
        [0; 8].as_slice(), &block_hash
    ].concat().try_into().unwrap();
    let mut bytes: [u8; 8];

    let mut i: usize;

    loop {
        bytes = blake2b_work(&data);
        bytes.reverse();
        if bytes >= difficulty {
            let mut work: [u8; 8] = data[..8].try_into().unwrap();
            work.reverse();
            return work
        }
        i = 0;
        loop {
            data[i] = data[i].wrapping_add(1);
            if data[i] != 0 {
                break;
            }
            i += 1;
        }
    }
}

/// Check if the given work is valid, given a difficulty target
pub fn check_work(work_hash: [u8; 32], difficulty: [u8; 8], work: [u8; 8]) -> bool {
    let mut work = work;
    work.reverse();

    let mut bytes = blake2b_work(&
        [work.as_slice(), &work_hash].concat()
    );
    bytes.reverse();

    bytes >= difficulty
}

/// Given a specific `r` value, sign the `message` with the `Key`, returning a `Signature`.
///
/// **DANGEROUS! Don't use unless you know what you're doing.**
pub fn sign_message_with_r(message: &[u8], private_key: &Key, r: &Scalar) -> Signature {
    let public_key = private_key.to_account().compressed.to_bytes();

    let r_point = r * G;
    let r_point_bytes = r_point.compress().to_bytes();

    let message = scalar!(blake2b512(
        &[&r_point_bytes, &public_key, message].concat()
    ));

    //s = r + H(m, pk, m)a
    let s = r + (message * private_key.as_scalar());

    Signature {
        r: r_point,
        s: s.as_ref().to_owned()
    }
}

/// Sign the `message` with the `Key`, returning a `Signature`.
///
/// This function does **not** produce identical signatures to the original Python `nanopy` library.
pub fn sign_message(message: &[u8], private_key: &Key) -> Signature {
    let r = blake2b_scalar(
        &[private_key.as_scalar().as_bytes(), message].concat()
    );
    sign_message_with_r(message, private_key, &r)
}

/// Check if the account's `signature` for the `message` is valid
pub fn is_valid_signature(message: &[u8], signature: Signature, public_key: &Account) -> bool {
    let r_bytes: [u8; 32] = signature.r.compress().to_bytes();
    let message = scalar!(blake2b512(
        &[r_bytes.as_slice(), public_key.compressed.as_bytes(), message].concat()
    ));

    //sG == R + H(m, pk, m)A
    signature.s * G == signature.r + (message * public_key.point)
}

pub(crate) fn hash_block(block: &Block) -> [u8; 32] {
    *blake2b256(&[
        [0; 31].as_slice(), &[6],
        block.account.compressed.as_bytes(),
        &block.previous,
        block.representative.compressed.as_bytes(),
        &block.balance.to_be_bytes(),
        &block.link
    ].concat()).as_ref()
}