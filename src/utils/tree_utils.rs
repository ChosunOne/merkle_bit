#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::constants::{KEY_LEN, KEY_LEN_BITS, MULTIPLY_DE_BRUIJN_BIT_POSITION};

/// This function checks if the given key should go down the zero branch at the given bit.
#[inline]
pub const fn choose_zero(key: &[u8; KEY_LEN], bit: u8) -> bool {
    let index = (bit >> 3) as usize;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    extracted_bit == 0
}

/// This function splits the list of sorted pairs into two lists, one for going down the zero branch,
/// and the other for going down the one branch.
#[inline]
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

/// This function checks to see if a section of keys need to go down this branch.
#[inline]
pub fn check_descendants<'a>(
    keys: &'a [&'a [u8; KEY_LEN]],
    branch_split_index: u8,
    branch_key: &[u8; KEY_LEN],
    min_split_index: u8,
) -> &'a [&'a [u8; KEY_LEN]] {
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

/// This function calculates the minimum index upon which the given keys diverge.  It also includes
/// the given branch key when calculating the minimum split index.
#[inline]
pub fn calc_min_split_index(keys: &[&[u8; KEY_LEN]], branch_key: &[u8; KEY_LEN]) -> u8 {
    assert!(!keys.is_empty());
    let mut min_key = *keys.iter().min().expect("Failed to get min key");
    let mut max_key = *keys.iter().max().expect("Failed to get max key");

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
    split_bit
}

/// This function initializes a hashmap to have entries for each provided key.  Values are initialized
/// to `None`.
#[inline]
pub fn generate_leaf_map<'a, ValueType>(
    keys: &[&'a [u8; KEY_LEN]],
) -> HashMap<&'a [u8; KEY_LEN], Option<ValueType>> {
    let mut leaf_map = HashMap::new();
    for &key in keys.iter() {
        leaf_map.insert(key, None);
    }
    leaf_map
}

/// This function performs a fast log2 operation for single byte unsigned integers.
#[inline]
pub const fn fast_log_2(num: u8) -> u8 {
    let mut log = num;
    log |= log >> 1;
    log |= log >> 2;
    log |= log >> 4;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[((0x1d_usize * log as usize) as u8 >> 5) as usize]
}
