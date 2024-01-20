// Copyright (c) 2021 gak

use bitvec::prelude::*;

const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
const ALPHABET_ARRAY: [char; 32] = [
    '1', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'w', 'x', 'y', 'z',
];

pub fn encode(bytes: &[u8]) -> String {
    let bits = bytes.view_bits::<Msb0>();
    assert_eq!(bits.len() % 5, 0);
    let mut s = String::new();
    for idx in (0..bits.len()).step_by(5) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + 5];
        let value: u8 = chunk.load_be();
        let char = ALPHABET_ARRAY[value as usize];
        s.push(char);
    }
    s
}

pub fn decode(s: &str) -> Option<Vec<u8>> {
    let mut bits: BitVec<Msb0, u8> = BitVec::new();
    for char in s.chars() {
        let value = ALPHABET.find(char)?;
        let value = value as u8;
        let char_bits: &BitSlice<Msb0, u8> = value.view_bits();
        bits.extend_from_bitslice(&char_bits[(8 - 5)..8]);
    }

    Some(bits.into_vec())
}

#[cfg(test)]
mod tests {
    const TEST_BYTES: [u8; 10] = [127, 255, 32, 8, 16, 50, 254, 0, 42, 96];
    const TEST_STR: &str = "hzzk141i8dz11cm1";

    #[test]
    fn encode() {
        assert!(super::encode(&TEST_BYTES) == TEST_STR)
    }
    #[test]
    fn decode() {
        assert!(super::decode(TEST_STR).unwrap() == TEST_BYTES.to_vec())
    }
}
