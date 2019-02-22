use std::collections::hash_map::{DefaultHasher, HashMap};
use std::error::Error;
use std::hash::Hasher;
use std::path::PathBuf;

#[cfg(feature = "default_tree")]
use bincode::{deserialize, serialize};

#[cfg(feature = "default_tree")]
use serde::{Serialize, Deserialize};

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT, NodeVariant};
use crate::traits::*;

#[derive(Clone)]
#[cfg_attr(feature = "default_tree", derive(Serialize, Deserialize))]
pub struct TreeBranch {
    count: u64,
    zero: Vec<u8>,
    one: Vec<u8>,
    split_index: u32,
    key: Vec<u8>,
}

impl TreeBranch {
    fn new() -> Self {
        Self {
            count: 0,
            zero: vec![],
            one: vec![],
            split_index: 0,
            key: vec![],
        }
    }

    fn get_count(&self) -> u64 {
        self.count
    }
    fn get_zero(&self) -> &[u8] {
        self.zero.as_ref()
    }
    fn get_one(&self) -> &[u8] {
        self.one.as_ref()
    }
    fn get_split_index(&self) -> u32 {
        self.split_index
    }
    fn get_key(&self) -> Option<&[u8]> { Some(&self.key) }

    fn set_count(&mut self, count: u64) {
        self.count = count;
    }
    fn set_zero(&mut self, zero: Vec<u8>) {
        self.zero = zero;
    }
    fn set_one(&mut self, one: Vec<u8>) {
        self.one = one;
    }
    fn set_split_index(&mut self, split_index: u32) {
        self.split_index = split_index;
    }
    fn set_key(&mut self, key: Vec<u8>) { self.key = key; }
}

impl Branch for TreeBranch {
    fn new() -> Self { Self::new() }

    fn get_count(&self) -> u64 { Self::get_count(&self) }
    fn get_zero(&self) -> &[u8] { Self::get_zero(&self) }
    fn get_one(&self) -> &[u8] { Self::get_one(&self) }
    fn get_split_index(&self) -> u32 { Self::get_split_index(&self) }
    fn get_key(&self) -> Option<&[u8]> { Self::get_key(&self) }

    fn set_count(&mut self, count: u64) { Self::set_count(self, count) }
    fn set_zero(&mut self, zero: &[u8]) { Self::set_zero(self, zero.to_vec()) }
    fn set_one(&mut self, one: &[u8]) { Self::set_one(self, one.to_vec()) }
    fn set_split_index(&mut self, index: u32) { Self::set_split_index(self, index) }
    fn set_key(&mut self, key: &[u8]) { Self::set_key(self, key.to_vec()) }
}

#[cfg(feature = "default_tree")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "default_tree")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "default_tree", derive(Serialize, Deserialize))]
pub struct TreeLeaf {
    key: Vec<u8>,
    data: Vec<u8>,
}

impl TreeLeaf {
    pub fn new() -> Self {
        Self {
            key: vec![],
            data: vec![],
        }
    }

    fn get_key(&self) -> &[u8] {
        self.key.as_ref()
    }
    fn get_data(&self) -> &[u8] {
        self.data.as_ref()
    }

    fn set_key(&mut self, key: Vec<u8>) {
        self.key = key;
    }
    fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

impl Leaf for TreeLeaf {
    fn new() -> Self { Self::new() }

    fn get_key(&self) -> &[u8] { Self::get_key(&self) }
    fn get_data(&self) -> &[u8] { Self::get_data(&self) }

    fn set_key(&mut self, key: &[u8]) { Self::set_key(self, key.to_vec()) }
    fn set_data(&mut self, data: &[u8]) { Self::set_data(self, data.to_vec()) }
}

#[cfg(feature = "default_tree")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "default_tree")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "default_tree", derive(Serialize, Deserialize))]
pub struct TreeData {
    value: Vec<u8>
}

impl TreeData {
    fn new() -> Self {
        Self {
            value: vec![]
        }
    }

    fn get_value(&self) -> &[u8] { self.value.as_ref() }

    fn set_value(&mut self, value: Vec<u8>) { self.value = value }
}

impl Data for TreeData {
    fn new() -> Self { Self::new() }

    fn get_value(&self) -> &[u8] { Self::get_value(&self) }

    fn set_value(&mut self, value: &[u8]) { Self::set_value(self, value.to_vec()) }
}

#[cfg(feature = "default_tree")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "default_tree")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "default_tree", derive(Serialize, Deserialize))]
pub struct TreeNode {
    references: u64,
    node: Option<NodeVariant<TreeBranch, TreeLeaf, TreeData>>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            references: 0,
            node: None,
        }
    }

    fn get_references(&self) -> u64 {
        self.references
    }

    fn set_references(&mut self, references: u64) {
        self.references = references;
    }
    fn set_branch(&mut self, branch: TreeBranch) {
        self.node = Some(NodeVariant::Branch(branch));
    }

    fn set_leaf(&mut self, leaf: TreeLeaf) {
        self.node = Some(NodeVariant::Leaf(leaf));
    }
    fn set_data(&mut self, data: TreeData) {
        self.node = Some(NodeVariant::Data(data));
    }
}

#[cfg(feature = "default_tree")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "default_tree")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

impl Node<TreeBranch, TreeLeaf, TreeData, Vec<u8>> for TreeNode {
    fn new() -> Self { Self::new() }

    fn get_references(&self) -> u64 { Self::get_references(&self) }
    fn get_variant(&self) -> BinaryMerkleTreeResult<NodeVariant<TreeBranch, TreeLeaf, TreeData>> {
        match self.node {
            Some(ref node_type) => {
                match node_type {
                    NodeVariant::Branch(branch) => Ok(NodeVariant::Branch(branch.clone())),
                    NodeVariant::Data(data) => Ok(NodeVariant::Data(data.clone())),
                    NodeVariant::Leaf(leaf) => Ok(NodeVariant::Leaf(leaf.clone()))
                }
            }
            None => Err(Box::new(Exception::new("Failed to distinguish node type")))
        }
    }

    fn set_references(&mut self, references: u64) { Self::set_references(self, references) }
    fn set_branch(&mut self, branch: TreeBranch) { Self::set_branch(self, branch) }
    fn set_leaf(&mut self, leaf: TreeLeaf) { Self::set_leaf(self, leaf) }
    fn set_data(&mut self, data: TreeData) { Self::set_data(self, data) }
}

impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self { Self::new() }
    fn update(&mut self, data: &[u8]) { Self::write(self, data) }
    fn finalize(self) -> Self::HashResultType { Self::finish(&self).to_le_bytes().to_vec() }
}

struct HashDB {
    map: HashMap<Vec<u8>, TreeNode>,
    pending_inserts: Vec<(Vec<u8>, TreeNode)>,
}

impl HashDB {
    pub fn new(map: HashMap<Vec<u8>, TreeNode>) -> Self {
        Self {
            map,
            pending_inserts: Vec::with_capacity(64),
        }
    }
}

impl Database for HashDB {
    type NodeType = TreeNode;
    type EntryType = (Vec<u8>, TreeNode);

    fn open(_path: &PathBuf) -> Result<Self, Box<Error>> { Ok(Self::new(HashMap::new())) }

    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
        if let Some(m) = self.map.get(key) {
            let node = m.clone();
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
        self.pending_inserts.push((key.to_vec(), value.clone()));
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        self.map.remove(key);
        Ok(())
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        while !self.pending_inserts.is_empty() {
            let entry = self.pending_inserts.remove(0);
            self.map.insert(entry.0, entry.1);
        }
        Ok(())
    }
}

pub struct HashTree {
    tree: MerkleBIT<HashDB, TreeBranch, TreeLeaf, TreeData, TreeNode, DefaultHasher, Vec<u8>, Vec<u8>>
}

impl HashTree {
    pub fn new(depth: usize) -> Self {
        let path = PathBuf::new();
        Self {
            tree: MerkleBIT::new(&path, depth).unwrap()
        }
    }

    pub fn get(&self, root_hash: &[u8], keys: Vec<&[u8]>) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&Vec<u8>>, keys: Vec<&[u8]>, values: &[&Vec<u8>]) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_fails_to_get_from_empty_hash_tree() {
        let key = vec![0x00];
        let root_key = vec![0x01];

        let bmt = HashTree::new(160);
        let items = bmt.get(&root_key, vec![&key[..]]).unwrap();
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item_from_hash_tree() {
        let key = vec![0xAAu8];
        let data = vec![0xFFu8];
        let values = vec![data.as_ref()];
        let mut bmt = HashTree::new(160);
        let root_hash = bmt.insert(None, vec![key.as_ref()], &values).unwrap();

        let nonexistent_key = vec![0xAB];
        let items = bmt.get(&root_hash, vec![&nonexistent_key[..]]).unwrap();
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_small_balanced_hash_tree() {
        let mut keys = Vec::with_capacity(8);
        let mut values = Vec::with_capacity(8);
        for i in 0..8 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = Vec::with_capacity(8);
        let mut get_data = Vec::with_capacity(8);
        for i in 0..8 {
            let key = &keys[i][..];
            get_keys.push(key);
            let data = &values[i];
            get_data.push(data);
        }

        let mut bmt = HashTree::new(3);
        let root_hash = bmt.insert(None, get_keys.clone(), &get_data).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in &values {
            expected_items.push(Some(value.clone()));
        }
        assert_eq!(items, expected_items);
    }
}