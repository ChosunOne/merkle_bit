use std::path::PathBuf;
use std::error::Error;
use std::fmt::Debug;
use std::cmp::{min, Ordering};
use std::marker::PhantomData;
use std::collections::VecDeque;
use std::iter::FromIterator;

#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use serde::{Serialize, Deserialize};

use crate::traits::{Encode, Exception, Decode, Branch, Data, Hasher, Database, Node, Leaf};

#[cfg(not(feature = "use_hashbrown"))]
use std::collections::HashMap;
#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

/// A generic Result from an operation involving a MerkleBIT
pub type BinaryMerkleTreeResult<T> = Result<T, Box<Error>>;

/// Contains the distinguishing data from the node
#[derive(Clone, Debug)]
#[cfg_attr(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"), derive(Serialize, Deserialize))]
pub enum NodeVariant<BranchType, LeafType, DataType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data {
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType),
}

#[derive(Debug, PartialEq)]
enum BranchSplit {
    Zero,
    One,
}

struct SplitPairs<'a> {
    zeros: &'a [&'a [u8]],
    ones: &'a [&'a [u8]],
}

struct TreeCell<'a, NodeType> {
    keys: &'a [&'a [u8]],
    node: Option<NodeType>,
    depth: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
struct TreeRef {
    key: Vec<u8>,
    location: Vec<u8>,
    count: u64,
}

impl Ord for TreeRef {
    fn cmp(&self, other_ref: &TreeRef) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}

impl<'a> SplitPairs<'a> {
    pub fn new(zeros: &'a [&'a [u8]], ones: &'a [&'a [u8]]) -> SplitPairs<'a> {
        SplitPairs {
            zeros,
            ones,
        }
    }
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(keys: &'a [&'a [u8]], node: Option<NodeType>, depth: usize) -> TreeCell<'a, NodeType>
        where BranchType: Branch,
              LeafType: Leaf,
              DataType: Data {
        TreeCell {
            keys,
            node,
            depth,
        }
    }
}

impl TreeRef {
    pub fn new(key: Vec<u8>, location: Vec<u8>, count: u64) -> TreeRef {
        TreeRef {
            key,
            location,
            count,
        }
    }
}

fn choose_branch(key: &[u8], bit: usize) -> BranchSplit {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    if extracted_bit == 0 {
        BranchSplit::Zero
    } else {
        BranchSplit::One
    }
}

fn split_pairs<'a>(sorted_pairs: &'a [&'a [u8]], bit: usize) -> SplitPairs {
    if sorted_pairs.is_empty() {
        return SplitPairs::new(&sorted_pairs[0..0], &sorted_pairs[0..0]);
    }

    if let BranchSplit::Zero = choose_branch(sorted_pairs[sorted_pairs.len() - 1], bit) {
        return SplitPairs::new(&sorted_pairs[0..sorted_pairs.len()], &sorted_pairs[0..0]);
    }

    if let BranchSplit::One = choose_branch(sorted_pairs[0], bit) {
        return SplitPairs::new(&sorted_pairs[0..0], &sorted_pairs[0..sorted_pairs.len()]);
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();

    while max - min > 1 {
        let bisect = (max - min) / 2 + min;
        match choose_branch(sorted_pairs[bisect], bit) {
            BranchSplit::Zero => min = bisect,
            BranchSplit::One => max = bisect
        }
    }

    let split = sorted_pairs.split_at(max);

    SplitPairs::new(split.0, split.1)
}

/// The MerkleBIT structure relies on many specified types:
/// # Required Type Annotations
/// * **DatabaseType**: The type to use for database-like operations.  DatabaseType must implement the Database trait.
/// * **BranchType**: The type used for representing branches in the tree.  BranchType must implement the Branch trait.
/// * **LeafType**: The type used for representing leaves in the tree.  LeafType must implement the Leaf trait.
/// * **DataType**: The type used for representing data nodes in the tree.  DataType must implement the Data trait.
/// * **NodeType**: The type used for the outer node that can be either a branch, leaf, or data.  NodeType must implement the Node trait.
/// * **HasherType**: The type of hasher to use for hashing locations on the tree.  HasherType must implement the Hasher trait.
/// * **HashResultType**: The type of the result from Hasher.  HashResultTypes must be able to be referenced as a &[u8] slice, and must implement basic traits
/// * **ValueType**: The type to return from a get.  ValueType must implement the Encode and Decode traits.
/// # Properties
/// * **db**: The database to store and retrieve values
/// * **depth**: The maximum permitted depth of the tree.
pub struct MerkleBIT<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, HashResultType, ValueType>
    where DatabaseType: Database<NodeType=NodeType>,
          BranchType: Branch,
          LeafType: Leaf + Clone,
          DataType: Data,
          NodeType: Node<BranchType, LeafType, DataType, ValueType>,
          HasherType: Hasher,
          HashResultType: AsRef<[u8]> + Clone + Eq + Debug + PartialOrd,
          ValueType: Decode + Encode {
    db: DatabaseType,
    depth: usize,
    branch: PhantomData<*const BranchType>,
    leaf: PhantomData<*const LeafType>,
    data: PhantomData<*const DataType>,
    node: PhantomData<*const NodeType>,
    hasher: PhantomData<*const HasherType>,
    hash_result: PhantomData<*const HashResultType>,
    value: PhantomData<*const ValueType>,
}

impl<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, HashResultType, ValueType>
MerkleBIT<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, HashResultType, ValueType>
    where DatabaseType: Database<NodeType=NodeType>,
          BranchType: Branch,
          LeafType: Leaf + Clone,
          DataType: Data,
          NodeType: Node<BranchType, LeafType, DataType, ValueType>,
          HasherType: Hasher<HashType=HasherType, HashResultType=HashResultType>,
          HashResultType: AsRef<[u8]> + Clone + Eq + Debug + PartialOrd,
          ValueType: Decode + Encode {
    /// Create a new MerkleBIT from a saved database
    pub fn new(path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = DatabaseType::open(path)?;
        Ok(Self {
            db,
            depth,
            branch: PhantomData,
            leaf: PhantomData,
            data: PhantomData,
            node: PhantomData,
            hasher: PhantomData,
            hash_result: PhantomData,
            value: PhantomData,
        })
    }

    /// Create a new MerkleBIT from an already opened database
    pub fn from_db(db: DatabaseType, depth: usize) -> BinaryMerkleTreeResult<Self> {
        Ok(Self {
            db,
            depth,
            branch: PhantomData,
            leaf: PhantomData,
            data: PhantomData,
            node: PhantomData,
            hasher: PhantomData,
            hash_result: PhantomData,
            value: PhantomData,
        })
    }

    /// Get items from the MerkleBIT.  Keys must be sorted.  Returns a list of Options which may include the corresponding values.
    pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<ValueType>>> {
        if keys.is_empty() {
            return Err(Box::new(Exception::new("Keys must not be empty")));
        }
        if keys[0].is_empty() {
            return Err(Box::new(Exception::new("Key size must be greater than 0")));
        }

        keys.sort();

        let root_node;
        if let Some(n) = self.db.get_node(root_hash)? {
            root_node = n;
        } else {
            let mut values = Vec::with_capacity(keys.len());
            for _ in 0..keys.len() {
                values.push(None);
            }
            return Ok(values);
        }

        let mut leaf_map = HashMap::new();

        let mut cell_queue = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);

        let root_cell = TreeCell::new::<BranchType, LeafType, DataType>(&keys, Some(root_node), 0);

        cell_queue.push_front(root_cell);

        while !cell_queue.is_empty() {
            let tree_cell;
            if let Some(c) = cell_queue.pop_front() {
                tree_cell = c;
            } else {
                return Err(Box::new(Exception::new("Empty cell queue")));
            }

            if tree_cell.depth > self.depth {
                return Err(Box::new(Exception::new("Depth of merkle tree exceeded")));
            }

            let node;
            match tree_cell.node {
                Some(n) => node = n,
                None => {
                    continue;
                }
            }

            match node.get_variant()? {
                NodeVariant::Branch(n) => {
                    let key_and_index = if n.get_key().is_some() {
                        self.calc_min_split_index(&tree_cell.keys, None, Some(&node))?
                    } else {
                        let mut hasher = HasherType::new(32);
                        hasher.update(b"b");
                        hasher.update(n.get_zero());
                        hasher.update(n.get_one());
                        let location = hasher.finalize();
                        self.calc_min_split_index(&tree_cell.keys, Some(location.as_ref()), None)?
                    };
                    let branch_key = key_and_index.0;
                    let min_split_index = key_and_index.1;
                    let descendants = Self::check_descendants(tree_cell.keys, &n, &branch_key, min_split_index);
                    if descendants.is_empty() {
                        continue;
                    }

                    let split = split_pairs(&descendants, n.get_split_index() as usize);

                    // If you switch the order of these blocks, the result comes out backwards
                    if let Some(o) = self.db.get_node(n.get_one())? {
                        let one_node = o;
                        if !split.ones.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, Some(one_node), tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    } else {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, None, tree_cell.depth);
                        cell_queue.push_front(new_cell);
                    }

                    if let Some(z) = self.db.get_node(n.get_zero())? {
                        let zero_node = z;
                        if !split.zeros.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.zeros, Some(zero_node), tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    }
                }
                NodeVariant::Leaf(n) => {
                    if tree_cell.keys.is_empty() {
                        return Err(Box::new(Exception::new("No key with which to match the leaf key")));
                    }

                    if let Some(d) = self.db.get_node(n.get_data())? {
                        if let NodeVariant::Data(data) = d.get_variant()? {
                            leaf_map.insert(n.get_key().to_owned(), data.get_value().to_owned());
                        } else {
                            return Err(Box::new(Exception::new("Corrupt merkle tree")));
                        }
                    } else {
                        return Err(Box::new(Exception::new("Corrupt merkle tree")));
                    }
                }
                NodeVariant::Data(_) => {
                    return Err(Box::new(Exception::new("Corrupt merkle tree")));
                }
            }
        }

        let mut values = Vec::with_capacity(keys.len());

        for key in keys {
            if let Some(v) = leaf_map.get(*key) {
                let val = ValueType::decode(v)?;
                values.push(Some(val));
            } else {
                values.push(None)
            }
        }

        Ok(values)
    }

    /// Insert items into the MerkleBIT.  Keys must be sorted.  Returns a new root hash for the MerkleBIT.
    pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&[u8]], values: &mut [&ValueType]) -> BinaryMerkleTreeResult<Vec<u8>> {
        if keys.len() != values.len() {
            return Err(Box::new(Exception::new("Keys and values have different lengths")));
        }

        if keys.is_empty() || values.is_empty() {
            return Err(Box::new(Exception::new("Keys or values are empty")));
        }

        {
            // Sort keys and values
            let mut value_map = HashMap::new();
            for i in 0..keys.len() {
                value_map.insert(keys[i], values[i]);
            }

            keys.sort();

            for i in 0..keys.len() {
                if let Some(v) = value_map.get(keys[i]) {
                    values[i] = *v;
                }
            }
        }

        let nodes = self.insert_leaves(keys, &&values[..])?;

        let mut tree_refs = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            let tree_ref = TreeRef::new(keys[i].to_vec(), nodes[i].as_ref().to_vec(), 1);
            tree_refs.push(tree_ref);
        }

        if let Some(n) = previous_root {
            // Nodes that form the merkle proof for the new tree
            let mut proof_nodes = Vec::with_capacity(keys.len());

            let root_node;
            if let Some(m) = self.db.get_node(n.as_ref())? {
                root_node = m;
            } else {
                return Err(Box::new(Exception::new("Could not find previous root")));
            }

            let mut cell_queue = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);
            let root_cell: TreeCell<NodeType> = TreeCell::new::<BranchType, LeafType, DataType>(&keys, Some(root_node), 0);
            cell_queue.push_front(root_cell);

            while !cell_queue.is_empty() {
                let tree_cell;
                if let Some(c) = cell_queue.pop_front() {
                    tree_cell = c;
                } else {
                    return Err(Box::new(Exception::new("Empty cell queue")));
                }

                if tree_cell.depth > self.depth {
                    return Err(Box::new(Exception::new("Depth of merkle tree exceeded")));
                }

                let mut node;
                if let Some(n) = tree_cell.node {
                    node = n;
                } else {
                    continue;
                }

                let branch;
                match node.get_variant()? {
                    NodeVariant::Branch(n) => {
                        branch = n
                    }
                    NodeVariant::Leaf(n) => {
                        let leaf = n;
                        let key = leaf.get_key();
                        let data = leaf.get_data();
                        let mut leaf_hasher = HasherType::new(32);
                        leaf_hasher.update(b"l");
                        leaf_hasher.update(key);
                        leaf_hasher.update(data);
                        let location = leaf_hasher.finalize();

                        let mut skip = false;
                        let mut old = false;

                        // Check if we are updating an existing value
                        for b in &tree_refs {
                            if b.key == key && b.location == location.as_ref().to_vec() {
                                // This value is not being updated, just update its reference count
                                old = true;
                                break;
                            } else if b.key == key {
                                // We are updating this value
                                skip = true;
                                break;
                            }
                        }

                        if skip {
                            continue;
                        }

                        if let Some(mut l) = self.db.get_node(location.as_ref())? {
                            let refs = l.get_references() + 1;
                            l.set_references(refs);
                            self.db.insert(location.as_ref(), &l)?;
                        } else {
                            return Err(Box::new(Exception::new("Corrupt merkle tree")));
                        }

                        if old {
                            continue;
                        }

                        let tree_ref = TreeRef::new(key.to_vec(), location.as_ref().to_vec(), 1);
                        proof_nodes.push(tree_ref);
                        continue;
                    }
                    NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }


                let mut branch_hasher = HasherType::new(32);
                branch_hasher.update(b"b");
                branch_hasher.update(branch.get_zero());
                branch_hasher.update(branch.get_one());
                let location = branch_hasher.finalize();
                let key_and_index = self.calc_min_split_index(&tree_cell.keys, Some(location.as_ref()), None)?;
                let branch_key = key_and_index.0;
                let min_split_index = key_and_index.1;

                let split;
                let mut descendants = &tree_cell.keys[..];

                if min_split_index < branch.get_split_index() as usize {
                    descendants = Self::check_descendants(&tree_cell.keys, &branch, &branch_key, min_split_index);

                    if descendants.is_empty() {
                        let tree_ref = TreeRef::new(branch_key, location.as_ref().to_vec(), branch.get_count());
                        let refs = node.get_references() + 1;
                        node.set_references(refs);
                        self.db.insert(location.as_ref(), &node)?;
                        proof_nodes.push(tree_ref);
                        continue;
                    }
                }

                split = split_pairs(descendants, branch.get_split_index() as usize);
                if let Some(o) = self.db.get_node(branch.get_one())? {
                    let mut one_node = o;
                    if !split.ones.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, Some(one_node), tree_cell.depth + 1);
                        cell_queue.push_front(new_cell);
                    } else {
                        let other_key = self.get_proof_key(Some(branch.get_one()), None)?;
                        assert!(!other_key.is_empty());
                        let count;
                        match one_node.get_variant()? {
                            NodeVariant::Branch(b) => count = b.get_count(),
                            NodeVariant::Leaf(_) => count = 1,
                            NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                        }
                        let tree_ref = TreeRef::new(other_key, branch.get_one().to_vec(), count);
                        let refs = one_node.get_references() + 1;
                        one_node.set_references(refs);
                        self.db.insert(branch.get_one(), &one_node)?;
                        proof_nodes.push(tree_ref);
                    }
                }
                if let Some(z) = self.db.get_node(branch.get_zero())? {
                    let mut zero_node = z;
                    if !split.zeros.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.zeros, Some(zero_node), tree_cell.depth + 1);
                        cell_queue.push_front(new_cell);
                    } else {
                        let other_key = self.get_proof_key(Some(branch.get_zero()), None)?;
                        assert!(!other_key.is_empty());
                        let count;
                        match zero_node.get_variant()? {
                            NodeVariant::Branch(b) => count = b.get_count(),
                            NodeVariant::Leaf(_) => count = 1,
                            NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                        }
                        let tree_ref = TreeRef::new(other_key, branch.get_zero().to_vec(), count);
                        let refs = zero_node.get_references() + 1;
                        zero_node.set_references(refs);
                        self.db.insert(branch.get_zero(), &zero_node)?;
                        proof_nodes.push(tree_ref);
                    }
                }
            }

            tree_refs.append(&mut proof_nodes);

            let new_root = self.create_tree(tree_refs)?;
            return Ok(new_root);
        } else {
            // There is no tree, just build one with the keys and values
            let new_root = self.create_tree(tree_refs)?;
            return Ok(new_root);
        }
    }

    fn check_descendants<'a>(keys: &'a [&'a [u8]], branch: &BranchType, branch_key: &[u8], min_split_index: usize) -> &'a [&'a [u8]] {
        // Check if any keys from the search need to go down this branch
        let mut start = 0;
        let mut end = 0;
        let mut found_start = false;
        for i in 0..keys.len() {
            let mut descendant = true;
            for j in min_split_index..branch.get_split_index() as usize {
                let left = choose_branch(&branch_key, j);
                let right = choose_branch(keys[i], j);
                if left != right {
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

    fn calc_min_split_index(&self, keys: &[&[u8]], location: Option<&[u8]>, node: Option<&NodeType>) -> BinaryMerkleTreeResult<(Vec<u8>, usize)> {
        let mut min_split_index = keys[0].len() * 8;
        let branch_key = self.get_proof_key(location, node)?;
        let mut all_keys = keys.to_owned();
        all_keys.push(branch_key.as_ref());
        for i in 0..all_keys.len() - 1 {
            for j in 0..all_keys[0].len() * 8 {
                let left = choose_branch(all_keys[i], j);
                let right = choose_branch(all_keys[i + 1], j);
                if left != right {
                    if j < min_split_index {
                        min_split_index = j;
                    }
                    break;
                }
            }
        }
        Ok((branch_key, min_split_index))
    }

    fn insert_leaves(&mut self, keys: &[&[u8]], values: &&[&ValueType]) -> BinaryMerkleTreeResult<Vec<HashResultType>> {
        let mut nodes: Vec<HashResultType> = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            // Create data node
            let mut data = DataType::new();
            data.set_value(&values[i].encode()?);

            let mut data_hasher = HasherType::new(32);
            data_hasher.update(b"d");
            data_hasher.update(keys[i]);
            data_hasher.update(data.get_value());
            let data_node_location = data_hasher.finalize();

            let mut data_node = NodeType::new();
            data_node.set_references(1);
            data_node.set_data(data);

            // Create leaf node
            let mut leaf = LeafType::new();
            leaf.set_data(data_node_location.as_ref());
            leaf.set_key(keys[i]);

            let mut leaf_hasher = HasherType::new(32);
            leaf_hasher.update(b"l");
            leaf_hasher.update(keys[i]);
            leaf_hasher.update(leaf.get_data());
            let leaf_node_location = leaf_hasher.finalize();

            let mut leaf_node = NodeType::new();
            leaf_node.set_references(1);
            leaf_node.set_leaf(leaf);

            if let Some(n) = self.db.get_node(data_node_location.as_ref())? {
                let references = n.get_references() + 1;
                data_node.set_references(references);
            }

            if let Some(n) = self.db.get_node(leaf_node_location.as_ref())? {
                let references = n.get_references() + 1;
                leaf_node.set_references(references);
            }

            self.db.insert(data_node_location.as_ref(), &data_node)?;
            self.db.insert(leaf_node_location.as_ref(), &leaf_node)?;

            nodes.push(leaf_node_location);
        }
        Ok(nodes)
    }

    fn create_tree(&mut self, mut tree_refs: Vec<TreeRef>) -> BinaryMerkleTreeResult<Vec<u8>> {
        tree_refs.sort();
        let mut tree_ref_queue: VecDeque<TreeRef> = VecDeque::from_iter(tree_refs);

        let mut split_indices = Vec::with_capacity(tree_ref_queue.len() - 1);
        for i in 0..tree_ref_queue.len() - 1 {
            let start_len = split_indices.len();
            assert!(!tree_ref_queue[i].key.is_empty());
            assert!(!tree_ref_queue[i + 1].key.is_empty());
            assert_ne!(tree_ref_queue[i].key, tree_ref_queue[i + 1].key);
            for j in 0..tree_ref_queue[i].key.len() * 8 {
                let left_branch = choose_branch(&tree_ref_queue[i].key, j);
                let right_branch = choose_branch(&tree_ref_queue[i + 1].key, j);

                if left_branch != right_branch {
                    split_indices.push(vec![i, j]);
                    break;
                } else if j == tree_ref_queue[i].key.len() * 8 - 1 {
                    // The keys are the same and don't diverge
                    return Err(Box::new(Exception::new("Attempted to insert item with duplicate keys")));
                }
            }

            assert_eq!(split_indices.len(), start_len + 1);
        }
        assert_eq!(split_indices.len(), tree_ref_queue.len() - 1);

        split_indices.sort_by(|a, b| {
            a[1].cmp(&b[1]).reverse()
        });

        while !tree_ref_queue.is_empty() {
            let start_len = tree_ref_queue.len();
            if tree_ref_queue.len() == 1 {
                self.db.batch_write()?;
                let root;
                if let Some(r) = tree_ref_queue.pop_front() {
                    root = r.location.to_vec();
                } else {
                    return Err(Box::new(Exception::new("Empty tree ref queue")))
                }
                return Ok(root);
            }

            let max_tree_ref = split_indices.remove(0);
            let max_index = max_tree_ref[0];

            for split_index in &mut split_indices {
                if split_index[0] > max_index {
                    split_index[0] -= 1;
                }
            }

            let tree_ref;
            if let Some(r) = tree_ref_queue.remove(max_index) {
                tree_ref = r;
            } else {
                return Err(Box::new(Exception::new("Empty tree ref queue")))
            }

            let next_tree_ref;
            if let Some(r) = tree_ref_queue.remove(max_index) {
                next_tree_ref = r;
            } else {
                return Err(Box::new(Exception::new("Empty tree ref queue")))
            }
            let mut branch_hasher = HasherType::new(32);
            branch_hasher.update(b"b");
            branch_hasher.update(tree_ref.location.as_ref());
            branch_hasher.update(next_tree_ref.location.as_ref());
            let branch_node_location = branch_hasher.finalize();

            let mut branch = BranchType::new();
            branch.set_zero(tree_ref.location.as_ref());
            branch.set_one(next_tree_ref.location.as_ref());
            let count = tree_ref.count + next_tree_ref.count;
            branch.set_count(count);
            branch.set_split_index(max_tree_ref[1] as u32);
            branch.set_key(min(&tree_ref.key, &next_tree_ref.key));

            let mut branch_node = NodeType::new();
            branch_node.set_branch(branch);
            branch_node.set_references(1);

            self.db.insert(branch_node_location.as_ref(), &branch_node)?;
            let new_tree_ref = TreeRef::new(tree_ref.key, branch_node_location.as_ref().to_vec(), count);
            tree_ref_queue.insert(max_index, new_tree_ref);
            assert_eq!(tree_ref_queue.len(), start_len - 1);
        }

        Err(Box::new(Exception::new("Corrupt merkle tree")))
    }

    fn get_proof_key(&self, root_hash: Option<&[u8]>, node: Option<&NodeType>) -> BinaryMerkleTreeResult<Vec<u8>> {
        if let Some(n) = node {
            match n.get_variant()? {
                NodeVariant::Branch(b) => {
                    if let Some(k) = b.get_key() {
                        return Ok(k.to_vec());
                    } else {
                        return Err(Box::new(Exception::new("Given node does not have a key")));
                    }
                }
                NodeVariant::Leaf(l) => {
                    return Ok(l.get_key().to_vec());
                }
                NodeVariant::Data(_) => { return Err(Box::new(Exception::new("Corrupt merkle tree"))); }
            }
        }

        let mut child_location;
        if let Some(h) = root_hash {
            child_location = h.to_vec();
        } else {
            return Err(Box::new(Exception::new("root_hash and node must not both be None")));
        }

        let mut key;

        let mut depth = 0;

        // DFS to find a key
        loop {
            if depth > self.depth {
                // If a poor hasher is chosen, you can end up with circular paths through the tree.
                // This check ensures that you are alerted of the possibility.
                return Err(Box::new(Exception::new("Maximum proof key depth exceeded.  Ensure hasher does not generate collisions.")));
            }
            if let Some(n) = self.db.get_node(child_location.as_ref())? {
                let node = n;
                match node.get_variant()? {
                    NodeVariant::Branch(m) => {
                        child_location = m.get_zero().to_owned();
                        if let Some(k) = m.get_key() {
                            return Ok(k.to_vec());
                        }
                    }
                    NodeVariant::Leaf(m) => {
                        key = m.get_key().to_vec();
                        return Ok(key);
                    }
                    NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }
            } else {
                return Err(Box::new(Exception::new("Corrupt merkle tree")));
            }
            depth += 1;
        }
    }

    /// Remove all items with less than 1 reference under the given root.
    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        let mut nodes = Vec::with_capacity(128);
        nodes.push(root_hash.to_vec());

        while !nodes.is_empty() {
            let node_location = nodes.remove(0);

            let mut node;
            if let Some(n) = self.db.get_node(&node_location)? {
                node = n;
            } else {
                continue;
            }

            let mut refs = node.get_references();
            if refs > 0 {
                refs -= 1;
            }

            match node.get_variant()? {
                NodeVariant::Branch(b) => {
                    if refs == 0 {
                        let zero = b.get_zero();
                        let one = b.get_one();
                        nodes.push(zero.to_vec());
                        nodes.push(one.to_vec());
                        self.db.remove(&node_location)?;
                        continue;
                    }
                }
                NodeVariant::Leaf(l) => {
                    if refs == 0 {
                        let data = l.get_data();
                        nodes.push(data.to_vec());
                        self.db.remove(&node_location)?;
                        continue;
                    }
                }
                NodeVariant::Data(_) => {
                    if refs == 0 {
                        self.db.remove(&node_location)?;
                        continue;
                    }
                }
            }

            node.set_references(refs);
            self.db.insert(&node_location, &node)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    extern crate rand;

    use super::*;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use crate::hash_tree::HashTree;

    #[test]
    fn it_chooses_the_right_branch_easy() {
        let key = vec![0x0F];
        for i in 0..8 {
            let expected_branch;
            if i < 4 {
                expected_branch = BranchSplit::Zero;
            } else {
                expected_branch = BranchSplit::One;
            }
            let branch = choose_branch(&key, i);
            assert_eq!(branch, expected_branch);
        }
    }

    #[test]
    fn it_chooses_the_right_branch_medium() {
        let key = vec![0x55];
        for i in 0..8 {
            let expected_branch;
            if i % 2 == 0 {
                expected_branch = BranchSplit::Zero;
            } else {
                expected_branch = BranchSplit::One;
            }
            let branch = choose_branch(&key, i);
            assert_eq!(branch, expected_branch);
        }
        let key = vec![0xAA];
        for i in 0..8 {
            let expected_branch;
            if i % 2 == 0 {
                expected_branch = BranchSplit::One;
            } else {
                expected_branch = BranchSplit::Zero;
            }
            let branch = choose_branch(&key, i);
            assert_eq!(branch, expected_branch);
        }
    }

    #[test]
    fn it_chooses_the_right_branch_hard() {
        let key = vec![0x68];
        for i in 0..8 {
            let expected_branch;
            if i == 1 || i == 2 || i == 4 {
                expected_branch = BranchSplit::One;
            } else {
                expected_branch = BranchSplit::Zero;
            }
            let branch = choose_branch(&key, i);
            assert_eq!(branch, expected_branch);
        }

        let key = vec![0xAB];
        for i in 0..8 {
            let expected_branch;
            if i == 0 || i == 2 || i == 4 || i == 6 || i == 7 {
                expected_branch = BranchSplit::One;
            } else {
                expected_branch = BranchSplit::Zero;
            }
            let branch = choose_branch(&key, i);
            assert_eq!(branch, expected_branch);
        }
    }

    #[test]
    fn it_splits_an_all_zeros_sorted_list_of_pairs() {
        // The complexity of these tests result from the fact that getting a key and splitting the
        // tree should not require any copying or moving of memory.
        let zero_key = vec![0x00u8];
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..]
        ];
        let keys = key_vec;

        let result = split_pairs(&keys, 0);
        assert_eq!(result.zeros.len(), 10);
        assert_eq!(result.ones.len(), 0);
        for i in 0..result.zeros.len() {
            assert_eq!(*result.zeros[i], [0x00u8]);
        }
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() {
        let one_key = vec![0xFFu8];
        let keys = vec![
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.zeros.len(), 0);
        assert_eq!(result.ones.len(), 10);
        for i in 0..result.ones.len() {
            assert_eq!(*result.ones[i], [0xFFu8]);
        }
    }

    #[test]
    fn it_splits_an_even_length_sorted_list_of_pairs() {
        let zero_key = vec![0x00u8];
        let one_key = vec![0xFFu8];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(*result.zeros[i], [0x00u8]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(*result.ones[i], [0xFFu8]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_zeros() {
        let zero_key = vec![0x00u8];
        let one_key = vec![0xFFu8];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.zeros.len(), 6);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(*result.zeros[i], [0x00u8]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(*result.ones[i], [0xFFu8]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_ones() {
        let zero_key = vec![0x00u8];
        let one_key = vec![0xFFu8];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];

        let result = split_pairs(&keys, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 6);
        for i in 0..result.zeros.len() {
            assert_eq!(*result.zeros[i], [0x00u8]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(*result.ones[i], [0xFFu8]);
        }
    }

    #[test]
    fn it_gets_an_item_out_of_a_simple_tree() -> BinaryMerkleTreeResult<()> {
        let key = vec![0xAAu8];
        let value = vec![0xFFu8];

        let mut bmt = HashTree::new(160);
        let root = bmt.insert(None, &mut [&key[..]], &mut vec![&value])?;
        let result = bmt.get(&root, &mut vec![&key[..]])?;
        assert_eq!(result, vec![Some(vec![0xFFu8])]);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_from_empty_tree() -> BinaryMerkleTreeResult<()> {
        let key = vec![0x00u8];
        let root_key = vec![0x01u8];

        let bmt = HashTree::new(160);
        let items = bmt.get(&root_key, &mut vec![&key[..]])?;
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item() -> BinaryMerkleTreeResult<()> {
        let key = vec![0xAAu8];
        let value = vec![0xFFu8];

        let mut bmt = HashTree::new(160);
        let root = bmt.insert(None, &mut [&key[..]], &mut vec![&value])?;

        let nonexistent_key = vec![0xAB];
        let items = bmt.get(&root, &mut vec![&nonexistent_key[..]])?;
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let mut keys = Vec::with_capacity(8);
        let mut values = Vec::with_capacity(8);
        for i in 0..8 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();

        let mut bmt = HashTree::new(3);
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(7);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(7);
        for i in 0..7 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let mut bmt = HashTree::new(3);

        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;
        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let num_leaves = 256;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(8);
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let num_leaves = 255;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(8);
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_balanced_tree() -> BinaryMerkleTreeResult<()> {
        #[cfg(not(any(feature = "use_groestl")))]
            let num_leaves = 8196;
        #[cfg(feature = "use_groestl")]
            let num_leaves = 1024;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        #[cfg(not(any(feature = "use_groestl")))]
            let num_leaves = 8195;
        #[cfg(feature = "use_groestl")]
            let num_leaves = 1023;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_complex_tree() -> BinaryMerkleTreeResult<()> {
        // Tree description
        // Node (Letter)
        // Key (Number)
        // Value (Number)
        //
        // A     B      C      D     E     F     G     H     I     J     K     L     M     N     O     P
        // 0x00  0x40, 0x41, 0x60, 0x68, 0x70, 0x71, 0x72, 0x80, 0xC0, 0xC1, 0xE0, 0xE1, 0xE2, 0xF0, 0xF8
        // None, None, None, 0x01, 0x02, None, None, None, 0x03, None, None, None, None, None, 0x04, None
        let pop_key_d = vec![0x60u8]; // 0110_0000   96 (Dec)
        let pop_key_e = vec![0x68u8]; // 0110_1000  104 (Dec)
        let pop_key_i = vec![0x80u8]; // 1000_0000  128 (Dec)
        let pop_key_o = vec![0xF0u8]; // 1111_0000  240 (Dec)

        let mut populated_keys = [&pop_key_d[..], &pop_key_e[..], &pop_key_i[..], &pop_key_o[..]];

        let pop_value_d = vec![0x01u8];
        let pop_value_e = vec![0x02u8];
        let pop_value_i = vec![0x03u8];
        let pop_value_o = vec![0x04u8];

        let mut populated_values = vec![&pop_value_d, &pop_value_e, &pop_value_i, &pop_value_o];

        let mut bmt = HashTree::new(5);
        let root_node = bmt.insert(None, &mut populated_keys, &mut populated_values)?;

        let key_a = vec![0x00u8]; // 0000_0000     0 (Dec)
        let key_b = vec![0x40u8]; // 0100_0000    64 (Dec)
        let key_c = vec![0x41u8]; // 0100_0001    65 (Dec)
        let key_f = vec![0x70u8]; // 0111_0000   112 (Dec)
        let key_g = vec![0x71u8]; // 0111_0001   113 (Dec)
        let key_h = vec![0x72u8]; // 0111_0010   114 (Dec)
        let key_j = vec![0xC0u8]; // 1100_0000   192 (Dec)
        let key_k = vec![0xC1u8]; // 1100_0001   193 (Dec)
        let key_l = vec![0xE0u8]; // 1110_0000   224 (Dec)
        let key_m = vec![0xE1u8]; // 1110_0001   225 (Dec)
        let key_n = vec![0xE2u8]; // 1110_0010   226 (Dec)
        let key_p = vec![0xF8u8]; // 1111_1000   248 (Dec)

        let mut keys = vec![
            &key_a[..], &key_b[..], &key_c[..], &pop_key_d[..],
            &pop_key_e[..], &key_f[..], &key_g[..], &key_h[..],
            &pop_key_i[..], &key_j[..], &key_k[..], &key_l[..],
            &key_m[..], &key_n[..], &pop_key_o[..], &key_p[..]];


        let items = bmt.get(&root_node, &mut keys)?;
        let expected_items = vec![
            None, None, None, Some(pop_value_d),
            Some(pop_value_e), None, None, None,
            Some(pop_value_i), None, None, None,
            None, None, Some(pop_value_o), None];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_returns_the_same_number_of_values_as_keys() -> BinaryMerkleTreeResult<()> {
        let initial_key = vec![0x00u8];
        let initial_value = vec![0xFFu8];

        let mut keys = Vec::with_capacity(256);
        for i in 0..256 {
            keys.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();

        let mut bmt = HashTree::new(3);
        let root_node = bmt.insert(None, &mut [&initial_key], &mut vec![&initial_value])?;

        let items = bmt.get(&root_node, &mut get_keys)?;
        let mut expected_items = vec![];
        for i in 0..256 {
            if i == 0 {
                expected_items.push(Some(vec![0xFF]));
            } else {
                expected_items.push(None);
            }
        }
        assert_eq!(items.len(), 256);
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let key = vec![0xAAu8];
        let data = vec![0xBBu8];

        let mut bmt = HashTree::new(3);
        let new_root_hash = bmt.insert(None, &mut [&key[..]], &mut vec![data.as_ref()])?;
        let items = bmt.get(&new_root_hash, &mut vec![&key[..]])?;
        let expected_items = vec![Some(vec![0xBBu8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let key_values = vec![
            vec![0x00u8],
            vec![0x01u8]
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(3);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree_with_first_bit_split() -> BinaryMerkleTreeResult<()> {
        let key_values = vec![
            vec![0x00u8],
            vec![0x80u8]
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(3);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let key_values = vec![
            vec![0xAAu8],  // 1010_1010
            vec![0xBBu8],  // 1011_1011
            vec![0xCCu8]]; // 1100_1100
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![vec![0xDDu8], vec![0xEEu8], vec![0xFFu8]];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(3);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0xDDu8]), Some(vec![0xEEu8]), Some(vec![0xFFu8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xAAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(32, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(31, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(255, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
            let prepare = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
            let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
            let prepare = prepare_inserts(4095, &mut rng);
        #[cfg(feature = "use_groestl")]
            let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = HashTree::new(16);
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_a_tree_with_one_item() -> BinaryMerkleTreeResult<()> {
        let first_key = vec![0xAAu8];
        let first_data = vec![0xBBu8];

        let second_key = vec![0xCCu8];
        let second_data = vec![0xDDu8];

        let mut bmt = HashTree::new(3);
        let new_root_hash = bmt.insert(None, &mut vec![first_key.as_ref()], &mut vec![first_data.as_ref()])?;
        let second_root_hash = bmt.insert(Some(&new_root_hash), &mut vec![second_key.as_ref()], &mut vec![second_data.as_ref()])?;

        let items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref()])?;
        let expected_items = vec![Some(vec![0xBBu8]), Some(vec![0xDDu8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_small_tree_with_existing_items() -> BinaryMerkleTreeResult<()> {
        let seed = [0x4d, 0x1b, 0xf8, 0xad, 0x2d, 0x5d, 0x2e, 0xcb, 0x59, 0x75, 0xc4, 0xb9,
            0x4d, 0xf9, 0xab, 0x5e, 0xf5, 0x12, 0xd4, 0x5c, 0x3d, 0xa0, 0x73, 0x4b,
            0x65, 0x5e, 0xc3, 0x82, 0xcb, 0x6c, 0xc0, 0x66];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let num_inserts = 2;
        let prepare_initial = prepare_inserts(num_inserts, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        let expected_initial_items = prepare_initial.2;
        let expected_added_items = prepare_added.2;

        assert_eq!(first_items, expected_initial_items);
        assert_eq!(second_items, expected_added_items);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_tree_with_existing_items() -> BinaryMerkleTreeResult<()> {
        let seed = [0xCAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
            let num_inserts = 4096;
        #[cfg(feature = "use_groestl")]
            let num_inserts = 256;
        let prepare_initial = prepare_inserts(num_inserts, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        let expected_initial_items = prepare_initial.2;
        let expected_added_items = prepare_added.2;

        assert_eq!(first_items, expected_initial_items);
        assert_eq!(second_items, expected_added_items);
        Ok(())
    }

    #[test]
    fn it_updates_an_existing_entry() -> BinaryMerkleTreeResult<()> {
        let key = vec![0xAAu8];
        let first_value = vec![0xBBu8];
        let second_value = vec![0xCCu8];

        let mut bmt = HashTree::new(3);
        let first_root_hash = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![first_value.as_ref()])?;
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut vec![key.as_ref()], &mut vec![second_value.as_ref()])?;

        let first_item = bmt.get(&first_root_hash, &mut vec![key.as_ref()])?;
        let expected_first_item = vec![Some(first_value.clone())];

        let second_item = bmt.get(&second_root_hash, &mut vec![key.as_ref()])?;
        let expected_second_item = vec![Some(second_value.clone())];

        assert_eq!(first_item, expected_first_item);
        assert_eq!(second_item, expected_second_item);
        Ok(())
    }

    #[test]
    fn it_updates_multiple_existing_entries() -> BinaryMerkleTreeResult<()> {
        let seed = [0xEEu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
            let prepare_initial = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
            let prepare_initial = prepare_inserts(256, &mut rng);

        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut updated_data_values = vec![];
        let mut updated_data = vec![];
        let mut expected_updated_data_values = vec![];
        for i in 0..initial_key_values.len() {
            let num = vec![i as u8; 32];
            updated_data_values.push(num.clone());
            expected_updated_data_values.push(Some(num));
        }

        for i in 0..initial_key_values.len() {
            updated_data.push(updated_data_values[i].as_ref());
        }

        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut initial_keys, &mut updated_data)?;

        let initial_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let updated_items = bmt.get(&second_root_hash, &mut initial_keys)?;

        let expected_initial_items = prepare_initial.2;
        assert_eq!(initial_items, expected_initial_items);
        assert_eq!(updated_items, expected_updated_data_values);
        Ok(())
    }

    #[test]
    fn it_does_not_panic_when_removing_a_nonexistent_node() -> BinaryMerkleTreeResult<()> {
        let mut bmt = HashTree::new(160);
        let missing_root_hash = vec![0x00u8];
        bmt.remove(&missing_root_hash)
    }

    #[test]
    fn it_removes_a_node() -> BinaryMerkleTreeResult<()> {
        let key = vec![0x00];
        let data = vec![0x01];

        let mut bmt = HashTree::new(160);
        let root_hash = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;

        let inserted_data = bmt.get(&root_hash, &mut vec![key.as_ref()])?;
        let expected_inserted_data = vec![Some(vec![0x01u8])];
        assert_eq!(inserted_data, expected_inserted_data);

        bmt.remove(&root_hash)?;

        let retrieved_values = bmt.get(&root_hash, &mut vec![key.as_ref()])?;
        let expected_retrieved_values = vec![None];
        assert_eq!(retrieved_values, expected_retrieved_values);
        Ok(())
    }

    #[test]
    fn it_removes_an_entire_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
            let prepare = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
            let prepare = prepare_inserts(256, &mut rng);

        let mut bmt = HashTree::new(160);
        let key_values = prepare.0;
        let data_values = prepare.1;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut data = data_values.iter().collect::<Vec<_>>();

        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let expected_inserted_items = prepare.2;
        let inserted_items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(inserted_items, expected_inserted_items);

        bmt.remove(&root_hash)?;
        let removed_items = bmt.get(&root_hash, &mut keys)?;
        let mut expected_removed_items = vec![];
        for _ in 0..keys.len() {
            expected_removed_items.push(None);
        }
        assert_eq!(removed_items, expected_removed_items);
        Ok(())
    }

    #[test]
    fn it_removes_an_old_root() -> BinaryMerkleTreeResult<()> {
        let first_key = vec![0x00u8];
        let first_data = vec![0x01u8];

        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut vec![first_key.as_ref()], &mut vec![first_data.as_ref()])?;

        let second_key = vec![0x02u8];
        let second_data = vec![0x03u8];

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut vec![second_key.as_ref()], &mut vec![second_data.as_ref()])?;
        bmt.remove(&first_root_hash)?;

        let retrieved_items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref()])?;
        let expected_retrieved_items = vec![Some(vec![0x01u8]), Some(vec![0x03u8])];
        assert_eq!(retrieved_items, expected_retrieved_items);
        Ok(())
    }

    #[test]
    fn it_removes_a_small_old_tree() -> BinaryMerkleTreeResult<()> {
        let first_key = vec![0x00u8];
        let second_key = vec![0x01u8];
        let third_key = vec![0x02u8];
        let fourth_key = vec![0x03u8];

        let first_data = vec![0x04u8];
        let second_data = vec![0x05u8];
        let third_data = vec![0x06u8];
        let fourth_data = vec![0x07u8];

        let mut first_keys = vec![first_key.as_ref(), second_key.as_ref()];
        let mut first_entries = vec![first_data.as_ref(), second_data.as_ref()];
        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut first_keys, &mut first_entries)?;

        let mut second_keys = vec![third_key.as_ref(), fourth_key.as_ref()];
        let mut second_entries = vec![third_data.as_ref(), fourth_data.as_ref()];
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut second_keys, &mut second_entries)?;
        bmt.remove(&first_root_hash)?;

        let items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref(), third_key.as_ref(), fourth_key.as_ref()])?;
        let expected_items = vec![Some(first_data.clone()), Some(second_data.clone()), Some(third_data.clone()), Some(fourth_data.clone())];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_removes_an_old_large_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0xBAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare_initial = prepare_inserts(16, &mut rng);
        let initial_key_values = prepare_initial.0;
        let initial_data_values = prepare_initial.1;
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = HashTree::new(160);
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(16, &mut rng);
        let added_key_values = prepare_added.0;
        let added_data_values = prepare_added.1;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let combined_size = initial_key_values.len() + added_key_values.len();
        let mut combined_keys = Vec::with_capacity(combined_size);
        let mut combined_expected_items = Vec::with_capacity(combined_size);
        let mut i = 0;
        let mut j = 0;
        for _ in 0..combined_size {
            if i == initial_key_values.len() {
                if j < added_key_values.len() {
                    combined_keys.push(added_key_values[j].as_ref());
                    combined_expected_items.push(prepare_added.2[j].clone());
                    j += 1;
                    continue;
                }
                continue;
            } else if j == added_key_values.len() {
                if i < initial_key_values.len() {
                    combined_keys.push(initial_key_values[i].as_ref());
                    combined_expected_items.push(prepare_initial.2[i].clone());
                    i += 1;
                    continue;
                }
                continue;
            }

            if i < initial_key_values.len() && initial_key_values[i] < added_key_values[j] {
                combined_keys.push(initial_key_values[i].as_ref());
                combined_expected_items.push(prepare_initial.2[i].clone());
                i += 1;
            } else if j < added_key_values.len() {
                combined_keys.push(added_key_values[j].as_ref());
                combined_expected_items.push(prepare_added.2[j].clone());
                j += 1;
            }
        }

        bmt.remove(&first_root_hash)?;
        let items = bmt.get(&second_root_hash, &mut combined_keys)?;
        assert_eq!(items, combined_expected_items);
        Ok(())
    }

    #[test]
    fn it_iterates_over_multiple_inserts_correctly() -> BinaryMerkleTreeResult<()> {
        let seed = [0xEFu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = HashTree::new(160);

        #[cfg(not(any(feature = "use_groestl")))]
            iterate_inserts(8, 100, &mut rng, &mut bmt)?;
        #[cfg(feature = "use_groestl")]
            iterate_inserts(8, 10, &mut rng, &mut bmt)?;
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_not_descendants() -> BinaryMerkleTreeResult<()> {
        let mut bmt = HashTree::new(160);

        let key_values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x10u8], vec![0x20u8]];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = values.iter().collect::<Vec<_>>();

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        let items = bmt.get(&second_root, &mut keys)?;
        let mut expected_items = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            expected_items.push(Some(values[i].clone()));
        }

        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_descendants() -> BinaryMerkleTreeResult<()> {
        let mut bmt = HashTree::new(160);

        let key_values = vec![vec![0x10u8], vec![0x11u8], vec![0x00u8], vec![0x01u8], vec![0x02u8]];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = values.iter().collect::<Vec<_>>();

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        keys.sort();

        let items = bmt.get(&second_root, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8]), Some(vec![0x04u8]), Some(vec![0x00u8]), Some(vec![0x01u8])];
        assert_eq!(items, expected_items);
        Ok(())
    }

    #[test]
    fn it_correctly_iterates_removals() -> BinaryMerkleTreeResult<()> {
        let seed = [0xA8u8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = HashTree::new(160);

        #[cfg(not(any(feature = "use_groestl")))]
            iterate_removals(8, 100, 1, &mut rng, &mut bmt)?;
        #[cfg(feature = "use_groestl")]
            iterate_removals(8, 10, 1, &mut rng, &mut bmt)?;
        Ok(())
    }

    #[test]
    fn it_correctly_increments_a_leaf_reference_count() -> BinaryMerkleTreeResult<()> {
        let mut bmt = HashTree::new(160);

        let key = vec![0x00u8];
        let data = vec![0x00u8];

        let first_root = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;
        let second_root = bmt.insert(Some(&first_root), &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;
        bmt.remove(&first_root)?;
        let item = bmt.get(&second_root, &mut vec![key.as_ref()])?;
        let expected_item = vec![Some(vec![0x00u8])];
        assert_eq!(item, expected_item);
        Ok(())
    }

    #[test]
    fn the_fuzz_test_function_works() -> BinaryMerkleTreeResult<()> {
        let rand_data = vec![];
        let data = &rand_data[..];
        let key_and_value = get_key_and_value(data);
        let mut key = key_and_value.0.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut val = key_and_value.1.iter().collect::<Vec<_>>();
        let mut bmt = HashTree::new(16);
        let root = bmt.insert(None, &mut key, &mut val)?;
        let items = bmt.get(&root, &mut key)?;
        assert_eq!(items, vec![Some(key_and_value.1[0].clone())]);
        Ok(())
    }

    fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Option<Vec<u8>>>) {
        let mut keys = Vec::with_capacity(num_entries);
        let mut data = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            let mut key_value = [0u8; 32];
            rng.fill(&mut key_value);
            keys.push(key_value.to_vec());

            let mut data_value = [0u8; 32];
            rng.fill(data_value.as_mut());
            data.push(data_value.to_vec());
        }
        let mut expected_items = vec![];
        for i in 0..num_entries {
            expected_items.push(Some(data[i].clone()));
        }

        keys.sort();

        (keys, data, expected_items)
    }

    fn iterate_inserts(entries_per_insert: usize,
                       iterations: usize,
                       rng: &mut StdRng,
                       bmt: &mut HashTree) -> BinaryMerkleTreeResult<(Vec<Option<Vec<u8>>>, Vec<Vec<Vec<u8>>>, Vec<Vec<Option<Vec<u8>>>>)> {
        let mut state_roots: Vec<Option<Vec<u8>>> = Vec::with_capacity(iterations);
        let mut key_groups = Vec::with_capacity(iterations);
        let mut data_groups = Vec::with_capacity(iterations);
        state_roots.push(None);

        for i in 0..iterations {
            let prepare = prepare_inserts(entries_per_insert, rng);
            let key_values = prepare.0;
            key_groups.push(key_values.clone());
            let data_values = prepare.1;
            let expected_data_values = prepare.2;
            data_groups.push(expected_data_values.clone());

            let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
            let mut data = data_values.iter().collect::<Vec<_>>();

            let previous_state_root = &state_roots[i].clone();
            let previous_root;
            match previous_state_root {
                Some(r) => previous_root = Some(r.as_slice()),
                None => previous_root = None
            }

            let new_root = bmt.insert(previous_root, &mut keys, &mut data)?;
            state_roots.push(Some(new_root.clone()));


            let retrieved_items = bmt.get(&new_root, &mut keys)?;
            assert_eq!(retrieved_items, expected_data_values);


            for j in 0..key_groups.len() {
                let mut key_block = Vec::with_capacity(key_groups[j].len());
                for k in 0..key_groups[j].len() {
                    key_block.push(key_groups[j][k].as_ref());
                }
                let items = bmt.get(&new_root, &mut key_block)?;
                assert_eq!(items, data_groups[j]);
            }
        }
        Ok((state_roots, key_groups, data_groups))
    }

    fn iterate_removals(entries_per_insert: usize,
                        iterations: usize,
                        removal_frequency: usize,
                        rng: &mut StdRng,
                        bmt: &mut HashTree) -> BinaryMerkleTreeResult<()> {
        let inserts = iterate_inserts(entries_per_insert, iterations, rng, bmt)?;
        let state_roots = inserts.0;
        let key_groups = inserts.1;
        let data_groups = inserts.2;

        for i in 1..iterations {
            if i % removal_frequency == 0 {
                let root;
                if let Some(r) = state_roots[i].clone() {
                    root = r.clone();
                } else {
                    panic!("state_roots[{}] is None", i);
                }
                bmt.remove(root.as_ref())?;
                for j in 0..iterations {
                    let mut keys = Vec::with_capacity(key_groups[i].len());
                    for k in 0..key_groups[i].len() {
                        keys.push(key_groups[i][k].as_ref());
                    }
                    let items = bmt.get(root.as_ref(), &mut keys)?;
                    let mut expected_items;
                    if j % removal_frequency == 0 {
                        expected_items = Vec::with_capacity(key_groups[i].len());
                        for _ in 0..key_groups[i].len() {
                            expected_items.push(None);
                        }
                    } else {
                        expected_items = data_groups[i].clone();
                    }
                    assert_eq!(items, expected_items);
                }
            }
        }
        Ok(())
    }

    fn get_key_and_value(data: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        if data.is_empty() || data.len() < 2 {
            return (vec![vec![0]], vec![vec![0]]);
        }
        let split = data.split_at(data.len() / 2);
        (vec![split.0.to_vec()], vec![split.1.to_vec()])
    }
}