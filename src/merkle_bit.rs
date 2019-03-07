use std::path::PathBuf;
use std::error::Error;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::collections::{BinaryHeap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use std::iter::FromIterator;

#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use serde::{Serialize, Deserialize};

use crate::traits::{Encode, exception, Decode, Branch, Data, Hasher, Database, Node, Leaf};

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

struct TreeCell<'a, NodeType> {
    keys: &'a [&'a [u8]],
    node: NodeType,
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

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(keys: &'a [&'a [u8]], node: NodeType, depth: usize) -> TreeCell<'a, NodeType>
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

fn choose_zero(key: &[u8], bit: usize) -> bool {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    extracted_bit == 0
}

fn split_pairs<'a>(sorted_pairs: &'a[&'a[u8]], bit: usize) -> (&'a[&'a[u8]], &'a[&'a[u8]]) {
    if sorted_pairs.is_empty() {
        return (&sorted_pairs[0..0], &sorted_pairs[0..0]);
    }

    if choose_zero(sorted_pairs[sorted_pairs.len() - 1], bit) {
        return (&sorted_pairs[0..sorted_pairs.len()], &sorted_pairs[0..0]);
    }

    if !choose_zero(sorted_pairs[0], bit) {
        return (&sorted_pairs[0..0], &sorted_pairs[0..sorted_pairs.len()]);
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();

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

/// Binary searches this sorted slice with a comparator function.
/// The comparator function should implement an order consistent with the sort order of the underlying slice,
/// returning an order code that indicates whether its argument is Less, Equal or Greater the desired target.
/// If the value is found then Result::Ok is returned, containing the index of the matching element.
/// If there are multiple matches, then any one of the matches could be returned.
/// If the value is not found then Result::Err is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
fn binary_search<T, F>(list: &VecDeque<T>, comparator: F) -> Result<usize, usize>
    where F: Fn(&T) -> Ordering {
    let mut size = list.len();
    if size == 0 {
        return Err(0);
    }

    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        let cmp = comparator(&list[mid]);
        base = if cmp == Ordering::Greater { base } else { mid };
        size -= half;
    }
    let cmp = comparator(&list[base]);
    if cmp == Ordering::Equal { Ok(base) } else { Err(base + (cmp == Ordering::Less) as usize) }
}

/// The MerkleBIT structure relies on many specified types:
/// # Required Type Annotations
/// * **DatabaseType**: The type to use for database-like operations.  DatabaseType must implement the Database trait.
/// * **BranchType**: The type used for representing branches in the tree.  BranchType must implement the Branch trait.
/// * **LeafType**: The type used for representing leaves in the tree.  LeafType must implement the Leaf trait.
/// * **DataType**: The type used for representing data nodes in the tree.  DataType must implement the Data trait.
/// * **NodeType**: The type used for the outer node that can be either a branch, leaf, or data.  NodeType must implement the Node trait.
/// * **HasherType**: The type of hasher to use for hashing locations on the tree.  HasherType must implement the Hasher trait.
/// * **ValueType**: The type to return from a get.  ValueType must implement the Encode and Decode traits.
/// # Properties
/// * **db**: The database to store and retrieve values
/// * **depth**: The maximum permitted depth of the tree.
pub struct MerkleBIT<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, ValueType>
    where DatabaseType: Database<NodeType=NodeType>,
          BranchType: Branch,
          LeafType: Leaf + Clone,
          DataType: Data,
          NodeType: Node<BranchType, LeafType, DataType, ValueType>,
          HasherType: Hasher,
          ValueType: Decode + Encode {
    db: DatabaseType,
    depth: usize,
    branch: PhantomData<*const BranchType>,
    leaf: PhantomData<*const LeafType>,
    data: PhantomData<*const DataType>,
    node: PhantomData<*const NodeType>,
    hasher: PhantomData<*const HasherType>,
    value: PhantomData<*const ValueType>,
}

impl<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, ValueType>
MerkleBIT<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, ValueType>
    where DatabaseType: Database<NodeType=NodeType>,
          BranchType: Branch,
          LeafType: Leaf + Clone,
          DataType: Data,
          NodeType: Node<BranchType, LeafType, DataType, ValueType>,
          HasherType: Hasher<HashType=HasherType>,
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
            value: PhantomData,
        })
    }

    /// Get items from the MerkleBIT.  Keys must be sorted.  Returns a list of Options which may include the corresponding values.
    pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<ValueType>>> {
        if keys.is_empty() {
            return Err(exception("Keys must not be empty"));
        }
        if keys[0].is_empty() {
            return Err(exception("Key size must be greater than 0"));
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

        let root_cell = TreeCell::new::<BranchType, LeafType, DataType>(&keys, root_node, 0);

        cell_queue.push_front(root_cell);

        while !cell_queue.is_empty() {
            let tree_cell;
            if let Some(c) = cell_queue.pop_front() {
                tree_cell = c;
            } else { unreachable!(); }

            if tree_cell.depth > self.depth {
                return Err(exception("Depth of merkle tree exceeded"));
            }

            let node = tree_cell.node;

            match node.get_variant() {
                NodeVariant::Branch(n) => {
                    let key_and_index = if n.get_key().is_some() {
                        self.calc_min_split_index(&tree_cell.keys, None, Some(&n))?
                    } else {
                        let mut hasher = HasherType::new(32);
                        hasher.update(b"b");
                        hasher.update(n.get_zero());
                        hasher.update(n.get_one());
                        let location = hasher.finalize();
                        self.calc_min_split_index(&tree_cell.keys, Some(&location), None)?
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
                        if !split.1.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.1, one_node, tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    }

                    if let Some(z) = self.db.get_node(n.get_zero())? {
                        let zero_node = z;
                        if !split.0.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.0, zero_node, tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    }
                }
                NodeVariant::Leaf(n) => {
                    if let Some(d) = self.db.get_node(n.get_data())? {
                        if let NodeVariant::Data(data) = d.get_variant() {
                            leaf_map.insert(n.get_key().to_owned(), data.get_value().to_owned());
                        } else {
                            return Err(exception("Corrupt merkle tree"));
                        }
                    } else {
                        return Err(exception("Corrupt merkle tree"));
                    }
                }
                NodeVariant::Data(_) => {
                    return Err(exception("Corrupt merkle tree"));
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
            return Err(exception("Keys and values have different lengths"));
        }

        if keys.is_empty() || values.is_empty() {
            return Err(exception("Keys or values are empty"));
        }

        {
            // Sort keys and values
            let mut value_map = HashMap::new();
            for (key, value) in keys.iter().zip(values.iter()) {
                value_map.insert(*key, *value);
            }

            keys.sort();

            for (key, value) in keys.iter().zip(values.iter_mut()) {
                if let Some(v) = value_map.get(key) {
                    *value = *v;
                }
            }
        }

        let nodes = self.insert_leaves(keys, &&values[..])?;

        let mut tree_refs = Vec::with_capacity(keys.len());
        for (loc, key) in nodes.into_iter().zip(keys.iter()) {
            let tree_ref = TreeRef::new(key.to_vec(), loc, 1);
            tree_refs.push(Rc::new(RefCell::new(tree_ref)));
        }

        if let Some(n) = previous_root {
            // Nodes that form the merkle proof for the new tree
            let mut proof_nodes = Vec::with_capacity(keys.len());

            let root_node = if let Some(m) = self.db.get_node(n.as_ref())? {
                m
            } else {
                return Err(exception("Could not find previous root"));
            };

            let mut cell_queue = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);
            let root_cell: TreeCell<NodeType> = TreeCell::new::<BranchType, LeafType, DataType>(&keys, root_node, 0);
            cell_queue.push_front(root_cell);

            while !cell_queue.is_empty() {
                let tree_cell = if let Some(c) = cell_queue.pop_front() {
                    c
                } else {
                    unreachable!();
                };

                if tree_cell.depth > self.depth {
                    return Err(exception("Depth of merkle tree exceeded"));
                }

                let node = tree_cell.node;

                let branch;
                let mut refs = node.get_references();
                match node.get_variant() {
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
                            let b_key = &RefCell::borrow(b).key;
                            let b_location = &RefCell::borrow(b).location;
                            if &b_key[..] == key && b_location == &location {
                                // This value is not being updated, just update its reference count
                                old = true;
                                break;
                            } else if &b_key[..] == key {
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
                            return Err(exception("Corrupt merkle tree"));
                        }

                        if old {
                            continue;
                        }

                        let tree_ref = TreeRef::new(key.to_vec(), location, 1);
                        proof_nodes.push(Rc::new(RefCell::new(tree_ref)));
                        continue;
                    }
                    NodeVariant::Data(_) => return Err(exception("Corrupt merkle tree"))
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
                        let tree_ref = TreeRef::new(branch_key, location, branch.get_count());
                        refs += 1;
                        let mut new_node = NodeType::new(NodeVariant::Branch(branch));
                        new_node.set_references(refs);
                        self.db.insert(&tree_ref.location, &new_node)?;
                        proof_nodes.push(Rc::new(RefCell::new(tree_ref)));
                        continue;
                    }
                }

                split = split_pairs(descendants, branch.get_split_index() as usize);
                if let Some(o) = self.db.get_node(branch.get_one())? {
                    let one_node = o;
                    if !split.1.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.1, one_node, tree_cell.depth + 1);
                        cell_queue.push_front(new_cell);
                    } else {
                        let other_key = self.get_proof_key(Some(branch.get_one()), None)?;
                        let count;
                        let refs = one_node.get_references() + 1;
                        let mut new_one_node ;
                        match one_node.get_variant() {
                            NodeVariant::Branch(b) => {
                                count = b.get_count();
                                new_one_node = NodeType::new(NodeVariant::Branch(b));
                            }
                            NodeVariant::Leaf(l) => {
                                count = 1;
                                new_one_node = NodeType::new(NodeVariant::Leaf(l));
                            }
                            NodeVariant::Data(_) => return Err(exception("Corrupt merkle tree"))
                        }
                        let tree_ref = TreeRef::new(other_key, branch.get_one().to_vec(), count);
                        new_one_node.set_references(refs);
                        self.db.insert(branch.get_one(), &new_one_node)?;
                        proof_nodes.push(Rc::new(RefCell::new(tree_ref)));
                    }
                }
                if let Some(z) = self.db.get_node(branch.get_zero())? {
                    let zero_node = z;
                    if !split.0.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.0, zero_node, tree_cell.depth + 1);
                        cell_queue.push_front(new_cell);
                    } else {
                        let other_key = self.get_proof_key(Some(branch.get_zero()), None)?;
                        let count;
                        let refs = zero_node.get_references() + 1;
                        let mut new_zero_node;
                        match zero_node.get_variant() {
                            NodeVariant::Branch(b) => {
                                count = b.get_count();
                                new_zero_node = NodeType::new(NodeVariant::Branch(b));
                            }
                            NodeVariant::Leaf(l) => {
                                count = 1;
                                new_zero_node = NodeType::new(NodeVariant::Leaf(l));
                            }
                            NodeVariant::Data(_) => return Err(exception("Corrupt merkle tree"))
                        }
                        let tree_ref = TreeRef::new(other_key, branch.get_zero().to_vec(), count);
                        new_zero_node.set_references(refs);
                        self.db.insert(branch.get_zero(), &new_zero_node)?;
                        proof_nodes.push(Rc::new(RefCell::new(tree_ref)));
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
                let left = choose_zero(&branch_key, j);
                let right = choose_zero(keys[i], j);
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

    fn calc_min_split_index(&self, keys: &[&[u8]], location: Option<&[u8]>, branch: Option<&BranchType>) -> BinaryMerkleTreeResult<(Vec<u8>, usize)> {
        let mut min_key;
        if let Some(m) = keys.iter().min() {
            min_key = *m;
        } else {
            return Err(exception("No keys to calculate minimum split index"));
        }

        let mut max_key;
        if let Some(m) = keys.iter().max() {
            max_key = *m;
        } else {
            return Err(exception("No keys to calculate minimum split index"));
        }

        let branch_key = self.get_proof_key(location, branch)?;

        if &branch_key[..] < min_key {
            min_key = &branch_key;
        } else if &branch_key[..] > max_key {
            max_key = &branch_key;
        }

        let mut split_bit = min_key.len() * 8;
        for i in 0..min_key.len() {
            if min_key[i] == max_key[i] {
                continue;
            }
            let xor_key = min_key[i] ^ max_key[i];
            split_bit = i * 8 + (7 - (xor_key as f32).log2().floor() as usize);
            break;
        }
        Ok((branch_key, split_bit))
    }

    fn insert_leaves(&mut self, keys: &[&[u8]], values: &&[&ValueType]) -> BinaryMerkleTreeResult<Vec<Vec<u8>>> {
        let mut nodes = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            // Create data node
            let mut data = DataType::new();
            data.set_value(&values[i].encode()?);

            let mut data_hasher = HasherType::new(32);
            data_hasher.update(b"d");
            data_hasher.update(keys[i]);
            data_hasher.update(data.get_value());
            let data_node_location = data_hasher.finalize();

            let mut data_node = NodeType::new(NodeVariant::Data(data));
            data_node.set_references(1);

            // Create leaf node
            let mut leaf = LeafType::new();
            leaf.set_data(data_node_location.as_ref());
            leaf.set_key(keys[i]);

            let mut leaf_hasher = HasherType::new(32);
            leaf_hasher.update(b"l");
            leaf_hasher.update(keys[i]);
            leaf_hasher.update(leaf.get_data());
            let leaf_node_location = leaf_hasher.finalize();

            let mut leaf_node = NodeType::new(NodeVariant::Leaf(leaf));
            leaf_node.set_references(1);

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

    fn create_tree(&mut self, mut tree_refs: Vec<Rc<RefCell<TreeRef>>>) -> BinaryMerkleTreeResult<Vec<u8>> {
        if tree_refs.is_empty() {
            return Err(exception("Tried to create a tree with no tree refs"));
        }
        tree_refs.sort();
        let mut tree_ref_queue: VecDeque<Rc<RefCell<TreeRef>>> = VecDeque::from_iter(tree_refs);

        let mut split_indices = BinaryHeap::with_capacity(tree_ref_queue.len() - 1);
        let keylen = RefCell::borrow(&tree_ref_queue[0]).key.len();
        for i in 0..tree_ref_queue.len() - 1 {
            let left_key = &RefCell::borrow(&tree_ref_queue[i]).key;
            let right_key = &RefCell::borrow(&tree_ref_queue[i + 1]).key;
            for j in 0..keylen {
                if j == keylen - 1 && left_key[j] == right_key[j] {
                    // The keys are the same and don't diverge
                    return Err(exception("Attempted to insert item with duplicate keys"));
                }
                // Skip bytes until we find a difference
                if left_key[j] == right_key[j] {
                    continue;
                }

                // Find the bit index of the first difference
                let xor_key = left_key[j] ^ right_key[j];
                let split_bit = j * 8 + (7 - ((xor_key as f32).log2().floor()) as usize);

                split_indices.push((split_bit, Rc::clone(&tree_ref_queue[i])));
                break;
            }
        }

        while !tree_ref_queue.is_empty() {
            if tree_ref_queue.len() == 1 {
                self.db.batch_write()?;
                let root;
                if let Some(c) = tree_ref_queue.pop_front() {
                    if let Ok(r) = Rc::try_unwrap(c) {
                        root = r.into_inner().location;
                    } else {
                        return Err(exception("Failed to return tree root"));
                    }
                } else {
                    return Err(exception("Failed to return tree root"));
                }
                return Ok(root);
            }

            let max_index;
            let split_index;
            {
                let max_tree_ref =
                    if let Some(s) = split_indices.pop() { s }
                    else { return Err(exception("Failed to get split index")); };

                max_index = binary_search(&tree_ref_queue, |x| {
                    if Rc::ptr_eq(&x, &max_tree_ref.1) {
                        return Ordering::Equal;
                    } else {
                        RefCell::borrow(x).key.cmp(&RefCell::borrow(&max_tree_ref.1).key)
                    }
                }).expect("Failed to find element in tree refs");
                split_index = max_tree_ref.0 as u32;
            }

            let mut branch = BranchType::new();
            let branch_node_location;
            let count;
            let tree_ref;
            let mut next_tree_ref;
            {
                tree_ref = tree_ref_queue.remove(max_index)
                    .expect("tree ref queue is empty when it shouldn't be");
                next_tree_ref = tree_ref_queue.remove(max_index)
                    .expect("tree ref queue is empty when it shouldn't be");

                let mut branch_hasher = HasherType::new(32);
                branch_hasher.update(b"b");
                branch_hasher.update(RefCell::borrow(&tree_ref).location.as_ref());
                branch_hasher.update(RefCell::borrow(&next_tree_ref).location.as_ref());
                branch_node_location = branch_hasher.finalize();


                let branch_key_ref = &RefCell::borrow(&tree_ref).key;
                count = &RefCell::borrow(&tree_ref).count + &RefCell::borrow(&next_tree_ref).count;
                branch.set_zero(&RefCell::borrow(&tree_ref).location.as_ref());
                branch.set_one(&RefCell::borrow(&next_tree_ref).location.as_ref());
                branch.set_count(count);
                branch.set_split_index(split_index);
                branch.set_key(branch_key_ref);
            }

            let unwrapped_tree_ref = Rc::try_unwrap(tree_ref)
                .expect("There are other references to the tree ref").into_inner();

            let mut branch_node = NodeType::new(NodeVariant::Branch(branch));
            branch_node.set_references(1);

            self.db.insert(branch_node_location.as_ref(), &branch_node)?;
            // Update reference of next_tree_ref in split_indices to point to the new_tree_ref
            next_tree_ref.borrow_mut().key = unwrapped_tree_ref.key;
            next_tree_ref.borrow_mut().location = branch_node_location.into();
            next_tree_ref.borrow_mut().count = count;

            tree_ref_queue.insert(max_index, Rc::clone(&next_tree_ref));
        }
        unreachable!();
    }

    fn get_proof_key(&self, root_hash: Option<&[u8]>, branch: Option<&BranchType>) -> BinaryMerkleTreeResult<Vec<u8>> {
        if let Some(b) = branch {
            if let Some(k) = b.get_key() {
                return Ok(k.to_vec());
            } else {
                return Err(exception("Given node does not have a key"));
            }
        }

        let child_location= if let Some(h) = root_hash {
            h
        } else {
            return Err(exception("root_hash and node must not both be None"));
        };

        let mut key;
        let mut depth = 0;
        let mut get_node = self.db.get_node(child_location)?;

        // DFS to find a key
        while let Some(node) = get_node {
            if depth > self.depth {
                // If a poor hasher is chosen, you can end up with circular paths through the tree.
                // This check ensures that you are alerted of the possibility.
                return Err(exception("Maximum proof key depth exceeded.  Ensure hasher does not generate collisions."));
            }
            let location;
                match node.get_variant() {
                    NodeVariant::Branch(m) => {
                        location = m.get_zero();
                        get_node = self.db.get_node(location)?;
                    }
                    NodeVariant::Leaf(m) => {
                        key = m.get_key().to_vec();
                        return Ok(key);
                    }
                    NodeVariant::Data(_) => return Err(exception("Corrupt merkle tree"))
                }
            depth += 1;
        }
        unreachable!();
    }

    /// Remove all items with less than 1 reference under the given root.
    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        let mut nodes = VecDeque::with_capacity(128);
        nodes.push_front(root_hash.to_vec());

        while !nodes.is_empty() {
            let node_location = if let Some(l) = nodes.pop_front() {
                l
            } else {
                return Err(exception("Empty node queue"));
            };

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

            let mut new_node;
            match node.get_variant() {
                NodeVariant::Branch(b) => {
                    if refs == 0 {
                        let zero = b.get_zero();
                        let one = b.get_one();
                        nodes.push_back(zero.to_vec());
                        nodes.push_back(one.to_vec());
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = NodeType::new(NodeVariant::Branch(b))
                }
                NodeVariant::Leaf(l) => {
                    if refs == 0 {
                        let data = l.get_data();
                        nodes.push_back(data.to_vec());
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = NodeType::new(NodeVariant::Leaf(l));
                }
                NodeVariant::Data(d) => {
                    if refs == 0 {
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = NodeType::new(NodeVariant::Data(d))
                }
            }

            new_node.set_references(refs);
            self.db.insert(&node_location, &new_node)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_chooses_the_right_branch_easy() {
        let key = vec![0x0F];
        for i in 0..8 {
            let expected_branch;
            if i < 4 {
                expected_branch = true;
            } else {
                expected_branch = false;
            }
            let branch = choose_zero(&key, i);
            assert_eq!(branch, expected_branch);
        }
    }

    #[test]
    fn it_chooses_the_right_branch_medium() {
        let key = vec![0x55];
        for i in 0..8 {
            let expected_branch;
            if i % 2 == 0 {
                expected_branch = true;
            } else {
                expected_branch = false;
            }
            let branch = choose_zero(&key, i);
            assert_eq!(branch, expected_branch);
        }
        let key = vec![0xAA];
        for i in 0..8 {
            let expected_branch;
            if i % 2 == 0 {
                expected_branch = false;
            } else {
                expected_branch = true;
            }
            let branch = choose_zero(&key, i);
            assert_eq!(branch, expected_branch);
        }
    }

    #[test]
    fn it_chooses_the_right_branch_hard() {
        let key = vec![0x68];
        for i in 0..8 {
            let expected_branch;
            if i == 1 || i == 2 || i == 4 {
                expected_branch = false;
            } else {
                expected_branch = true;
            }
            let branch = choose_zero(&key, i);
            assert_eq!(branch, expected_branch);
        }

        let key = vec![0xAB];
        for i in 0..8 {
            let expected_branch;
            if i == 0 || i == 2 || i == 4 || i == 6 || i == 7 {
                expected_branch = false;
            } else {
                expected_branch = true;
            }
            let branch = choose_zero(&key, i);
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
        assert_eq!(result.0.len(), 10);
        assert_eq!(result.1.len(), 0);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8]);
        }
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() {
        let one_key = vec![0xFFu8];
        let keys = vec![
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 0);
        assert_eq!(result.1.len(), 10);
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8]);
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
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 5);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8]);
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
        assert_eq!(result.0.len(), 6);
        assert_eq!(result.1.len(), 5);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8]);
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
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 6);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8]);
        }
    }
}