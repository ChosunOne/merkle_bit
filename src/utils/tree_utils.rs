#[cfg(not(any(feature = "hashbrown")))]
use std::collections::hash_map::Entry;
#[cfg(not(any(feature = "hashbrown")))]
use std::collections::HashMap;

#[cfg(feature = "hashbrown")]
use hashbrown::hash_map::Entry;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

use crate::constants::MULTIPLY_DE_BRUIJN_BIT_POSITION;
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Exception;
use crate::utils::tree_ref::TreeRef;
use std::convert::TryFrom;

use crate::Array;
#[cfg(feature = "hashbrown")]
use hashbrown::HashSet;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashSet;

/// This function checks if the given key should go down the zero branch at the given bit.
/// # Errors
/// `Exception` generated from a failure to convert an `u8` to an `usize`
#[inline]
pub fn choose_zero<const N: usize>(key: Array<N>, bit: usize) -> Result<bool, Exception> {
    let index = bit >> 3_usize;
    let shift = bit % 8;
    if let Some(v) = key.get(index) {
        let extracted_bit = usize::try_from(*v)? >> (7 - shift) & 1;
        return Ok(extracted_bit == 0);
    }
    Err(Exception::new("Designated bit exceeds key length"))
}

/// This function splits the list of sorted pairs into two lists, one for going down the zero branch,
/// and the other for going down the one branch.
/// # Errors
/// `Exception` generated from a failure to convert an `u8` to an `usize`
#[inline]
pub fn split_pairs<const N: usize>(
    sorted_pairs: &[Array<N>],
    bit: usize,
) -> Result<(&[Array<N>], &[Array<N>]), Exception> {
    if sorted_pairs.is_empty() {
        return Ok((&[], &[]));
    }

    if let Some(&last) = sorted_pairs.last() {
        if choose_zero(last, bit)? {
            return Ok((sorted_pairs, &[]));
        }
    }

    if let Some(&first) = sorted_pairs.first() {
        if !choose_zero(first, bit)? {
            return Ok((&[], sorted_pairs));
        }
    }

    let pp = sorted_pairs.partition_point(|&v| {
        if let Ok(b) = choose_zero(v, bit) {
            b
        } else {
            false
        }
    });

    Ok(sorted_pairs.split_at(pp))
}

/// This function checks to see if a section of keys need to go down this branch.
/// # Errors
/// `Exception` generated from a failure to convert an `u8` to an `usize`
#[inline]
pub fn check_descendants<'keys, const N: usize>(
    keys: &'keys [Array<N>],
    branch_split_index: usize,
    branch_key: &Array<N>,
    min_split_index: usize,
) -> Result<&'keys [Array<N>], Exception> {
    let mut start = 0;
    let mut end = 0;
    let mut found_start = false;
    for (i, k) in keys.iter().enumerate() {
        let key = k.as_ref();
        let mut descendant = true;
        for j in (min_split_index..branch_split_index).step_by(8) {
            let byte = j >> 3_usize;
            if branch_key[byte] == key[byte] {
                continue;
            }
            let xor_key: u8 = branch_key[byte] ^ key[byte];
            let split_bit = (byte << 3_usize) + 7 - usize::try_from(fast_log_2(xor_key))?;
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
    Ok(&keys[start..end])
}

/// This function calculates the minimum index upon which the given keys diverge.  It also includes
/// the given branch key when calculating the minimum split index.
/// # Errors
/// May return an `Exception` if the supplied `keys` is empty.
#[inline]
pub fn calc_min_split_index<const N: usize>(
    keys: &[Array<N>],
    branch_key: &Array<N>,
) -> Result<usize, Exception> {
    let mut min_key = if let Some(key) = keys.first() {
        key
    } else {
        return Err(Exception::new("Failed to get min key from list of keys."));
    };
    let mut max_key = if let Some(key) = keys.last() {
        key
    } else {
        return Err(Exception::new("Failed to get max key from list of keys."));
    };

    if branch_key < min_key {
        min_key = branch_key;
    } else if branch_key > max_key {
        max_key = branch_key;
    }

    let mut split_bit = N * 8 - 1;
    for (i, &min_key_byte) in min_key.iter().enumerate() {
        if min_key_byte == max_key[i] {
            continue;
        }
        let xor_key: u8 = min_key_byte ^ max_key[i];
        split_bit = (i << 3_usize) + 7_usize - usize::try_from(fast_log_2(xor_key))?;
        break;
    }
    Ok(split_bit)
}

/// This function initializes a hashmap to have entries for each provided key.  Values are initialized
/// to `None`.
#[inline]
#[must_use]
pub fn generate_leaf_map<ValueType, const N: usize>(
    keys: &[Array<N>],
) -> HashMap<Array<N>, Option<ValueType>> {
    let mut leaf_map = HashMap::new();
    for &key in keys {
        leaf_map.insert(key, None);
    }
    leaf_map
}

/// This function performs a fast log2 operation for single byte unsigned integers.
#[inline]
#[must_use]
pub const fn fast_log_2(num: u8) -> u8 {
    let mut log = num;
    log |= log >> 1_u8;
    log |= log >> 2_u8;
    log |= log >> 4_u8;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[((0x1d_usize * log as usize) as u8 >> 5_u8) as usize]
}

/// Generates the `TreeRef`s that will be made into the new tree.
/// # Errors
/// `Exception` generated from a failure to convert a `u8` to a `usize`
#[inline]
pub fn generate_tree_ref_queue<S: std::hash::BuildHasher, const N: usize>(
    tree_refs: &mut Vec<TreeRef<N>>,
    tree_ref_queue: &mut HashMap<usize, Vec<(usize, usize, usize)>, S>,
) -> BinaryMerkleTreeResult<HashSet<usize>> {
    let mut unique_split_bits = HashSet::new();
    for i in 0..tree_refs.len() - 1 {
        let left_key = tree_refs[i].key.as_ref();
        let right_key = tree_refs[i + 1].key.as_ref();
        let key_len = left_key.len();

        for j in 0..key_len {
            if j == key_len - 1_usize && left_key[j] == right_key[j] {
                // The keys are the same and don't diverge
                return Err(Exception::new(
                    "Attempted to insert item with duplicate keys",
                ));
            }
            // Skip bytes until we find a difference
            if left_key[j] == right_key[j] {
                continue;
            }

            // Find the bit index of the first difference
            let xor_key: u8 = left_key[j] ^ right_key[j];
            let split_bit = (j * 8_usize) + 7_usize - usize::try_from(fast_log_2(xor_key))?;
            unique_split_bits.insert(split_bit);
            let new_item = (split_bit, i, i + 1_usize);
            match tree_ref_queue.entry(split_bit) {
                Entry::Occupied(o) => (*o.into_mut()).push(new_item),
                Entry::Vacant(v) => {
                    v.insert(vec![new_item]);
                }
            };

            break;
        }
    }
    Ok(unique_split_bits)
}
