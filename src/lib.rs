// Clippy configurations
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::implicit_return)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::mod_module_files)]
#![allow(clippy::separated_literal_suffix)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![forbid(unsafe_code)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::semicolon_outside_block)]

//! # Merkle Binary Indexed Tree
//! ## Introduction
//! This module implements [`MerkleBIT`](merkle_bit/struct.MerkleBIT.html) with an attached storage module.  The implemented [`HashTree`](hash_tree/struct.HashTree.html)
//! and [`RocksTree`](rocks_tree/struct.RocksTree.html) structures allow use with persistence in memory and storage respectively.  Write
//! operations are batched together and committed at the end of each insert op.  The [`MerkleBit`](merkle_bit/struct.MerkleBIT.html) API
//! abstracts all actions related to maintaining and updating the storage tree.  The public APIs are
//! * [`new`](merkle_bit/struct.MerkleBIT.html#method.new)
//! * [`from_db`](merkle_bit/struct.MerkleBIT.html#method.from_db)
//! * [`get`](merkle_bit/struct.MerkleBIT.html#method.get)
//! * [`insert`](merkle_bit/struct.MerkleBIT.html#method.insert)
//! * [`remove`](merkle_bit/struct.MerkleBIT.html#method.remove)
//! * [`generate_inclusion_proof`](merkle_bit/struct.MerkleBIT.html#method.generate_inclusion_proof)
//! * [`get_one`](merkle_bit/struct.MerkleBIT.html#method.get_one)
//! * [`insert_one`](merkle_bit/struct.MerkleBIT.html#method.insert_one)
//! * and the associated function [`verify_inclusion_proof`](merkle_bit/struct.MerkleBIT.html#method.verify_inclusion_proof).
//!
//! After each call to either `insert` or `insert_one`, a new root hash will be created which can be
//! later used to access the inserted items.
//!
//! ## Internal Structure
//! Internally, the `MerkleBit` is composed of a collection of trait structs which implement the
//! [`Branch`](traits/trait.Branch.html), [`Leaf`](traits/trait.Leaf.html), and [`Data`](traits/trait.Data.html) nodes of the tree.
//!
//! A `Branch` node contains first a `split_index`
//! which indicates which bit of a given hash should be used to traverse the tree.  This is an optimisation
//! that makes this a *sparse* merkle tree.  It then contains pointers
//! to either the `one` side of the tree or the `zero` side of the tree.  Additionally, a branch contains
//! a copy of the `key` used during creation to determine if a branch should be inserted before it, and
//! a `count` of the nodes under that branch.
//!
//! A `Leaf` node contains an associated `key` for comparison, and a pointer to a `Data` node for retrieving
//! information regarding access to the data.  This is separate from the `Data` node for the purpose of only
//! accessing data information if data should be retrieved.
//!
//! A `Data` node contains the actual information to be retrieved.  `Data` nodes can be arbitrary in size
//! and the only restriction is that the data must be serializable and deserializable.
//!
//! To illustrate these concepts, please refer to the diagram below:
//!
//! ```text
//!                                                 ----------------------
//!                                 branch  --->    | split_index: usize |
//!                                   |             | zero: [u8]         |
//!       ----------------           / \            | one:  [u8]         |
//!       | key: [u8]    |          /   \           | count: u64         |
//!       | data: [u8]   | <----  leaf   leaf       | key:   [u8]        |
//!       ----------------         |                ----------------------
//!                                |
//!                                V
//!                              data
//!                                |
//!                                V
//!                         ------------------
//!                         | value: Vec<u8> |
//!                         ------------------
//! ```
//!
//! The `MerkleBIT` can be extended to support a wide variety of backend storage solutions given that
//! you make implementations for the `Branch`, `Leaf`, and `Data` traits.

#[cfg(feature = "serde")]
use serde::de::{Error, Visitor};
#[cfg(feature = "serde")]
use serde::ser::SerializeSeq;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};
#[cfg(feature = "serde")]
use std::array::IntoIter;
#[cfg(feature = "serde")]
use std::cmp::min;
#[cfg(feature = "serde")]
use std::fmt::Formatter;
#[cfg(feature = "serde")]
use std::ops::{Deref, DerefMut, Index, IndexMut};
#[cfg(feature = "serde")]
use std::slice::{Iter, SliceIndex};

/// Defines constants for the tree.
pub mod constants;
/// An implementation of the `MerkleBIT` with a `HashMap` backend database.
pub mod hash_tree;
/// Contains the actual operations of inserting, getting, and removing items from a tree.
pub mod merkle_bit;
/// Contains the traits necessary for tree operations
pub mod traits;
/// Contains a collection of structs for representing locations within the tree.
pub mod tree;
/// Contains a collection of structs for implementing tree databases.
pub mod tree_db;
/// Contains a collection of structs for implementing hashing functions in the tree.
pub mod tree_hasher;
/// Contains a collection of useful structs and functions for tree operations.
pub mod utils;

/// The prelude for the crate
pub mod prelude;
#[cfg(feature = "rocksdb")]
/// An implementation of the `MerkleBIT` with a `RocksDB` backend database.
pub mod rocks_tree;

/// Alias for a fixed sized array
#[cfg(not(any(feature = "serde")))]
pub type Array<const N: usize> = [u8; N];

/// A fixed-size array.  Needed because not all of the serialization libraries can handle arbitrary
/// sized arrays.  Can be converted to and from a `[u8; N]` via `into` and `from`.
#[cfg(feature = "serde")]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Array<const N: usize>([u8; N]);

#[cfg(feature = "serde")]
impl<const N: usize> Array<N> {
    /// Produces an iterator through the underlying array.
    #[inline]
    pub fn iter(&self) -> Iter<u8> {
        self.0.iter()
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> Default for Array<N> {
    #[inline]
    fn default() -> Self {
        Self([0; N])
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> From<[u8; N]> for Array<N> {
    #[inline]
    fn from(array: [u8; N]) -> Self {
        Self(array)
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> From<Array<N>> for [u8; N] {
    #[inline]
    fn from(array: Array<N>) -> Self {
        array.0
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> IntoIterator for Array<N> {
    type Item = u8;
    type IntoIter = IntoIter<u8, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> AsRef<[u8]> for Array<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> Deref for Array<N> {
    type Target = [u8; N];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> DerefMut for Array<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serde")]
impl<const N: usize, Idx: SliceIndex<[u8]>> Index<Idx> for Array<N> {
    type Output = Idx::Output;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(feature = "serde")]
impl<const N: usize, Idx: SliceIndex<[u8]>> IndexMut<Idx> for Array<N> {
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> Serialize for Array<N> {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(N))?;
        for e in self.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
/// Visitor for deserializing `Array<N>`s.
struct ArrayVisitor<const N: usize>;

#[cfg(feature = "serde")]
impl<'de, const N: usize> Visitor<'de> for ArrayVisitor<N> {
    type Value = Array<N>;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an unsigned integer from 0 to 255")
    }

    #[inline]
    fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        let mut value = Array::default();
        for i in 0..(min(N, v.len())) {
            value[i] = v[i];
        }

        Ok(value)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> Deserialize<'de> for Array<N> {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(ArrayVisitor)
    }
}
