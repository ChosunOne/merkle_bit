#[cfg(not(feature = "use_hashbrown"))]
use std::collections::HashMap;
#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;
#[cfg(feature = "use_rayon")]
use rayon::prelude::*;

use crate::constants::{KEY_LEN, KEY_LEN_BITS};
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Exception;

pub fn choose_zero(key: &[u8; KEY_LEN], bit: u8) -> bool {
    let index = (bit >> 3) as usize;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    extracted_bit == 0
}

pub fn split_pairs<'a>(
    sorted_pairs: &'a [&'a [u8; KEY_LEN]],
    bit: u8,
) -> (&'a [&'a [u8; KEY_LEN]], &'a [&'a [u8; KEY_LEN]]) {
    if sorted_pairs.is_empty() {
        return (&[], &[]);
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();

    if choose_zero(sorted_pairs[max - 1], bit) {
        return (&sorted_pairs[..], &[]);
    }

    if !choose_zero(sorted_pairs[0], bit) {
        return (&[], &sorted_pairs[..]);
    }

    while max - min > 1 {
        let bisect = (max - min) / 2 + min;
        if choose_zero(sorted_pairs[bisect], bit) {
            min = bisect;
        } else {
            max = bisect;
        }
    }

    sorted_pairs.split_at(max)
}

pub fn check_descendants<'a>(
    keys: &'a [&'a [u8; KEY_LEN]],
    branch_split_index: u8,
    branch_key: &[u8; KEY_LEN],
    min_split_index: u8,
) -> &'a [&'a [u8; KEY_LEN]] {
    // Check if any keys from the search need to go down this branch
    let mut start = 0;
    let mut end = 0;
    let mut found_start = false;
    for (i, key) in keys.iter().enumerate() {
        let mut descendant = true;
        for j in (min_split_index..branch_split_index).step_by(8) {
            let byte = (j >> 3) as usize;
            if branch_key[byte] == key[byte] {
                continue;
            }
            let xor_key = branch_key[byte] ^ key[byte];
            let split_bit = (byte << 3) as u8 + (7 - fast_log_2(xor_key)) as u8;
            if split_bit < branch_split_index {
                descendant = false;
                break;
            }
        }
        if descendant && !found_start {
            start = i;
            found_start = true;
        }
        if !descendant && found_start {
            end = i;
            break;
        }
        if descendant && i == keys.len() - 1 && found_start {
            end = i + 1;
            break;
        }
    }
    &keys[start..end]
}

pub fn calc_min_split_index(
    keys: &[&[u8; KEY_LEN]],
    branch_key: &[u8; KEY_LEN],
) -> BinaryMerkleTreeResult<u8> {
    let mut min_key;
    if let Some(&m) = keys.iter().min() {
        min_key = m;
    } else {
        return Err(Exception::new("No keys to calculate minimum split index"));
    }

    let mut max_key = if let Some(&m) = keys.iter().max() {
        m
    } else {
        return Err(Exception::new("No keys to calculate minimum split index"));
    };

    if branch_key < min_key {
        min_key = branch_key;
    } else if branch_key > max_key {
        max_key = branch_key;
    }

    let mut split_bit = KEY_LEN_BITS;
    for (i, &min_key_byte) in min_key.iter().enumerate() {
        if min_key_byte == max_key[i] {
            continue;
        }
        let xor_key = min_key_byte ^ max_key[i];
        split_bit = (i << 3) as u8 + (7 - fast_log_2(xor_key)) as u8;
        break;
    }
    Ok(split_bit)
}

pub fn generate_leaf_map<'a, ValueType>(keys: &[&'a [u8; KEY_LEN]]) -> HashMap<&'a [u8; KEY_LEN], Option<ValueType>> {
    let mut leaf_map = HashMap::new();
    for &key in keys.iter() {
        leaf_map.insert(key, None);
    }
    leaf_map
}

pub fn fast_log_2(num: u8) -> u8 {
    let mut log = num;
    log |= log >> 1;
    log |= log >> 2;
    log |= log >> 4;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[((0x1dusize * log as usize) as u8 >> 5) as usize]
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 8] = [0, 5, 1, 6, 4, 3, 2, 7];
