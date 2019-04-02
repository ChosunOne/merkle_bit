use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;

#[cfg(any(
    feature = "use_serde",
    feature = "use_bincode",
    feature = "use_json",
    feature = "use_cbor",
    feature = "use_yaml",
    feature = "use_pickle",
    feature = "use_ron"
))]
use serde::{Deserialize, Serialize};

use crate::traits::{Branch, Data, Database, Decode, Encode, Exception, Hasher, Leaf, Node};

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "use_hashbrown"))]
use std::collections::HashMap;

/// A generic Result from an operation involving a MerkleBIT
pub type BinaryMerkleTreeResult<T> = Result<T, Exception>;

/// Contains the distinguishing data from the node
#[derive(Clone, Debug)]
#[cfg_attr(
    any(
        feature = "use_serde",
        feature = "use_bincode",
        feature = "use_json",
        feature = "use_cbor",
        feature = "use_yaml",
        feature = "use_pickle",
        feature = "use_ron"
    ),
    derive(Serialize, Deserialize)
)]
pub enum NodeVariant<BranchType, LeafType, DataType>
where
    BranchType: Branch,
    LeafType: Leaf,
    DataType: Data,
{
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType),
}

struct TreeCell<'a, NodeType> {
    keys: &'a [&'a [u8]],
    node: NodeType,
    depth: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
struct TreeRef {
    key: Rc<[u8; 32]>,
    location: Rc<[u8; 32]>,
    count: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TreeRefWrapper {
    raw: Option<Rc<RefCell<TreeRef>>>,
    reference: Option<Rc<RefCell<TreeRefWrapper>>>,
}

impl TreeRefWrapper {
    pub fn new(tree_ref: Rc<RefCell<TreeRef>>) -> Self {
        Self {
            raw: Some(tree_ref),
            reference: None,
        }
    }

    pub fn update_reference(&mut self) {
        if self.raw.is_some() {
            return;
        }
        let mut new_ref;
        if let Some(r) = &self.reference {
            if r.borrow().raw.is_some() {
                return;
            }
            new_ref = r.borrow().get_reference();
        } else {
            unreachable!();
        }
        self.reference.replace(new_ref);
    }

    pub fn set_reference(&mut self, other: Rc<RefCell<TreeRefWrapper>>) {
        self.raw = None;
        if other.borrow().raw.is_some() {
            self.reference = Some(other);
        } else {
            self.reference = Some(other.borrow().get_reference());
        }
    }

    pub fn get_reference(&self) -> Rc<RefCell<TreeRefWrapper>> {
        if let Some(r) = &self.reference {
            if r.borrow().raw.is_some() {
                return Rc::clone(r);
            } else {
                return r.borrow().get_reference();
            }
        }
        unreachable!();
    }

    pub fn get_tree_ref_key(&self) -> Rc<[u8; 32]> {
        if let Some(t) = &self.raw {
            return Rc::clone(&t.borrow().key);
        }
        if let Some(r) = &self.reference {
            return r.borrow().get_tree_ref_key();
        }
        unreachable!();
    }

    pub fn get_tree_ref_location(&self) -> Rc<[u8; 32]> {
        if let Some(t) = &self.raw {
            return Rc::clone(&t.borrow().location);
        }
        if let Some(r) = &self.reference {
            return r.borrow().get_tree_ref_location();
        }
        unreachable!();
    }

    pub fn get_tree_ref_count(&self) -> u64 {
        if let Some(t) = &self.raw {
            return t.borrow().count;
        }
        if let Some(r) = &self.reference {
            return r.borrow().get_tree_ref_count();
        }
        unreachable!();
    }

    pub fn set_tree_ref_key(&mut self, key: Rc<[u8; 32]>) {
        if let Some(t) = &mut self.raw {
            t.borrow_mut().key = key;
        } else if let Some(r) = &mut self.reference {
            r.borrow_mut().set_tree_ref_key(key);
        } else {
            unreachable!();
        }
    }

    pub fn set_tree_ref_location(&mut self, location: Rc<[u8; 32]>) {
        if let Some(t) = &mut self.raw {
            t.borrow_mut().location = location;
        } else if let Some(r) = &mut self.reference {
            r.borrow_mut().set_tree_ref_location(location);
        } else {
            unreachable!();
        }
    }

    pub fn set_tree_ref_count(&mut self, count: u64) {
        if let Some(t) = &mut self.raw {
            t.borrow_mut().count = count;
        } else if let Some(r) = &mut self.reference {
            r.borrow_mut().set_tree_ref_count(count);
        } else {
            unreachable!();
        }
    }
}

impl Ord for TreeRef {
    fn cmp(&self, other_ref: &TreeRef) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(
        keys: &'a [&'a [u8]],
        node: NodeType,
        depth: usize,
    ) -> TreeCell<'a, NodeType>
    where
        BranchType: Branch,
        LeafType: Leaf,
        DataType: Data,
    {
        TreeCell { keys, node, depth }
    }
}

impl TreeRef {
    pub fn new(key: [u8; 32], location: [u8; 32], count: u64) -> TreeRef {
        TreeRef {
            key: Rc::new(key),
            location: Rc::new(location),
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

fn split_pairs<'a>(sorted_pairs: &'a [&'a [u8]], bit: usize) -> (&'a [&'a [u8]], &'a [&'a [u8]]) {
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
where
    DatabaseType: Database<NodeType = NodeType>,
    BranchType: Branch,
    LeafType: Leaf + Clone,
    DataType: Data,
    NodeType: Node<BranchType, LeafType, DataType, ValueType>,
    HasherType: Hasher,
    ValueType: Decode + Encode,
{
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
where
    DatabaseType: Database<NodeType = NodeType>,
    BranchType: Branch,
    LeafType: Leaf + Clone,
    DataType: Data,
    NodeType: Node<BranchType, LeafType, DataType, ValueType>,
    HasherType: Hasher<HashType = HasherType>,
    ValueType: Decode + Encode,
{
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

    /// Get items from the MerkleBIT.  Returns a map of Options which may include the corresponding values.
    pub fn get<'a>(
        &self,
        root_hash: &[u8],
        keys: &mut [&'a [u8]],
    ) -> BinaryMerkleTreeResult<HashMap<&'a [u8], Option<ValueType>>> {
        let mut leaf_map = HashMap::new();
        for key in keys.iter() {
            leaf_map.insert(*key, None);
        }

        if keys.is_empty() {
            return Ok(leaf_map);
        }
        if keys[0].is_empty() {
            return Err(Exception::new("Key size must be greater than 0"));
        }
        if keys.iter().any(|&x| x.len() != 32) {
            return Err(Exception::new("Key size must be 32 bytes"));
        }
        keys.sort();

        let root_node;
        if let Some(n) = self.db.get_node(root_hash)? {
            root_node = n;
        } else {
            return Ok(leaf_map);
        }

        let mut cell_queue = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);

        let root_cell = TreeCell::new::<BranchType, LeafType, DataType>(&keys, root_node, 0);

        cell_queue.push_front(root_cell);

        while !cell_queue.is_empty() {
            let tree_cell;
            if let Some(c) = cell_queue.pop_front() {
                tree_cell = c;
            } else {
                unreachable!();
            }

            if tree_cell.depth > self.depth {
                return Err(Exception::new("Depth of merkle tree exceeded"));
            }

            let node = tree_cell.node;

            match node.get_variant() {
                NodeVariant::Branch(branch) => {
                    let (_, zero, one, branch_split_index, branch_key) = branch.deconstruct();
                    let min_split_index =
                        self.calc_min_split_index(&tree_cell.keys, &branch_key)?;
                    let descendants = Self::check_descendants(
                        tree_cell.keys,
                        branch_split_index as usize,
                        &branch_key,
                        min_split_index,
                    );
                    if descendants.is_empty() {
                        continue;
                    }

                    let split = split_pairs(&descendants, branch_split_index as usize);

                    // If you switch the order of these blocks, the result comes out backwards
                    if let Some(o) = self.db.get_node(&one)? {
                        let one_node = o;
                        if !split.1.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                                split.1,
                                one_node,
                                tree_cell.depth + 1,
                            );
                            cell_queue.push_front(new_cell);
                        }
                    }

                    if let Some(z) = self.db.get_node(&zero)? {
                        let zero_node = z;
                        if !split.0.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                                split.0,
                                zero_node,
                                tree_cell.depth + 1,
                            );
                            cell_queue.push_front(new_cell);
                        }
                    }
                }
                NodeVariant::Leaf(n) => {
                    if let Some(d) = self.db.get_node(n.get_data())? {
                        if let NodeVariant::Data(data) = d.get_variant() {
                            let value = ValueType::decode(data.get_value())?;
                            if let Ok(index) = keys.binary_search(&&n.get_key()[..]) {
                                leaf_map.insert(keys[index], Some(value));
                            }
                        } else {
                            return Err(Exception::new("Corrupt merkle tree"));
                        }
                    } else {
                        return Err(Exception::new("Corrupt merkle tree"));
                    }
                }
                NodeVariant::Data(_) => {
                    return Err(Exception::new("Corrupt merkle tree"));
                }
            }
        }

        Ok(leaf_map)
    }

    /// Insert items into the MerkleBIT.  Keys must be sorted.  Returns a new root hash for the MerkleBIT.
    pub fn insert(
        &mut self,
        previous_root: Option<&[u8]>,
        keys: &mut [&[u8]],
        values: &mut [&ValueType],
    ) -> BinaryMerkleTreeResult<[u8; 32]> {
        if keys.len() != values.len() {
            return Err(Exception::new("Keys and values have different lengths"));
        }

        if keys.is_empty() || values.is_empty() {
            return Err(Exception::new("Keys or values are empty"));
        }

        if keys.iter().any(|&x| x.len() != 32) {
            return Err(Exception::new("Key size must be 32 bytes"));
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

        let nodes = self.insert_leaves(keys, &values[..])?;

        let mut tree_refs = Vec::with_capacity(keys.len());
        for (loc, key) in nodes.into_iter().zip(keys.iter()) {
            let mut tree_ref_key = [0; 32];
            tree_ref_key.copy_from_slice(key);
            let tree_ref = TreeRef::new(tree_ref_key, loc, 1);
            tree_refs.push(tree_ref);
        }

        if let Some(n) = previous_root {
            // Nodes that form the merkle proof for the new tree
            let mut proof_nodes = Vec::with_capacity(keys.len());

            let root_node = if let Some(m) = self.db.get_node(n)? {
                m
            } else {
                return Err(Exception::new("Could not find previous root"));
            };

            let mut cell_queue = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);
            let root_cell: TreeCell<NodeType> =
                TreeCell::new::<BranchType, LeafType, DataType>(&keys, root_node, 0);
            cell_queue.push_front(root_cell);

            while !cell_queue.is_empty() {
                let tree_cell = cell_queue
                    .pop_front()
                    .expect("cell queue should not be empty");

                if tree_cell.depth > self.depth {
                    return Err(Exception::new("Depth of merkle tree exceeded"));
                }

                let node = tree_cell.node;

                let branch;
                let mut refs = node.get_references();
                match node.get_variant() {
                    NodeVariant::Branch(n) => branch = n,
                    NodeVariant::Leaf(n) => {
                        let (key, data) = n.deconstruct();

                        let mut leaf_hasher = HasherType::new(32);
                        leaf_hasher.update(b"l");
                        leaf_hasher.update(&key);
                        leaf_hasher.update(&data);
                        let mut location = [0; 32];
                        location.copy_from_slice(&leaf_hasher.finalize());

                        let mut skip = false;
                        let mut old = false;

                        // Check if we are updating an existing value
                        if let Ok(index) = tree_refs.binary_search_by(|x| x.key[..].cmp(&key)) {
                            if tree_refs[index].location[..] == location {
                                old = true;
                            } else {
                                skip = true;
                            }
                        }

                        if skip {
                            continue;
                        }

                        if let Some(mut l) = self.db.get_node(&location)? {
                            let refs = l.get_references() + 1;
                            l.set_references(refs);
                            self.db.insert(location.as_ref(), &l)?;
                        } else {
                            return Err(Exception::new("Corrupt merkle tree"));
                        }

                        if old {
                            continue;
                        }

                        let tree_ref = TreeRef::new(key, location, 1);
                        proof_nodes.push(tree_ref);
                        continue;
                    }
                    _ => return Err(Exception::new("Corrupt merkle tree")),
                }

                let (branch_count, branch_zero, branch_one, branch_split_index, branch_key) =
                    branch.deconstruct();

                let mut branch_hasher = HasherType::new(32);
                branch_hasher.update(b"b");
                branch_hasher.update(&branch_zero);
                branch_hasher.update(&branch_one);
                let location_vec = branch_hasher.finalize();
                let mut location = [0; 32];
                location.clone_from_slice(&location_vec);
                drop(location_vec);

                let min_split_index = self.calc_min_split_index(&tree_cell.keys, &branch_key)?;

                let split;
                let mut descendants = &tree_cell.keys[..];

                if min_split_index < branch_split_index as usize {
                    descendants = Self::check_descendants(
                        &tree_cell.keys,
                        branch_split_index as usize,
                        &branch_key,
                        min_split_index,
                    );

                    if descendants.is_empty() {
                        let mut new_branch = BranchType::new();
                        new_branch.set_count(branch_count);
                        new_branch.set_zero(branch_zero);
                        new_branch.set_one(branch_one);
                        new_branch.set_split_index(branch_split_index);
                        new_branch.set_key(branch_key);

                        let tree_ref = TreeRef::new(branch_key, location, branch_count);
                        refs += 1;
                        let mut new_node = NodeType::new(NodeVariant::Branch(new_branch));
                        new_node.set_references(refs);
                        self.db.insert(&tree_ref.location[..], &new_node)?;
                        proof_nodes.push(tree_ref);
                        continue;
                    }
                }

                split = split_pairs(descendants, branch_split_index as usize);
                if let Some(o) = self.db.get_node(&branch_one)? {
                    let one_node = o;
                    if !split.1.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                            split.1,
                            one_node,
                            tree_cell.depth + 1,
                        );
                        cell_queue.push_front(new_cell);
                    } else {
                        let mut other_key = [0; 32];
                        let count;
                        let refs = one_node.get_references() + 1;
                        let mut new_one_node;
                        match one_node.get_variant() {
                            NodeVariant::Branch(b) => {
                                count = b.get_count();
                                other_key.copy_from_slice(b.get_key());
                                new_one_node = NodeType::new(NodeVariant::Branch(b));
                            }
                            NodeVariant::Leaf(l) => {
                                count = 1;
                                other_key.copy_from_slice(l.get_key());
                                new_one_node = NodeType::new(NodeVariant::Leaf(l));
                            }
                            NodeVariant::Data(_) => {
                                return Err(Exception::new("Corrupt merkle tree"));
                            }
                        }
                        new_one_node.set_references(refs);
                        self.db.insert(&branch_one, &new_one_node)?;
                        let tree_ref = TreeRef::new(other_key, branch_one, count);
                        proof_nodes.push(tree_ref);
                    }
                }
                if let Some(z) = self.db.get_node(&branch_zero)? {
                    let zero_node = z;
                    if !split.0.is_empty() {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                            split.0,
                            zero_node,
                            tree_cell.depth + 1,
                        );
                        cell_queue.push_front(new_cell);
                    } else {
                        let mut other_key = [0; 32];
                        let count;
                        let refs = zero_node.get_references() + 1;
                        let mut new_zero_node;
                        match zero_node.get_variant() {
                            NodeVariant::Branch(b) => {
                                count = b.get_count();
                                other_key.copy_from_slice(b.get_key());
                                new_zero_node = NodeType::new(NodeVariant::Branch(b));
                            }
                            NodeVariant::Leaf(l) => {
                                count = 1;
                                other_key.copy_from_slice(l.get_key());
                                new_zero_node = NodeType::new(NodeVariant::Leaf(l));
                            }
                            NodeVariant::Data(_) => {
                                return Err(Exception::new("Corrupt merkle tree"));
                            }
                        }
                        new_zero_node.set_references(refs);
                        self.db.insert(&branch_zero, &new_zero_node)?;
                        let tree_ref = TreeRef::new(other_key, branch_zero, count);
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

    fn check_descendants<'a>(
        keys: &'a [&'a [u8]],
        branch_split_index: usize,
        branch_key: &[u8],
        min_split_index: usize,
    ) -> &'a [&'a [u8]] {
        // Check if any keys from the search need to go down this branch
        let mut start = 0;
        let mut end = 0;
        let mut found_start = false;
        for i in 0..keys.len() {
            let mut descendant = true;
            for j in (min_split_index..branch_split_index).step_by(8) {
                let byte = j / 8;
                if branch_key[byte] == keys[i][byte] {
                    continue;
                }
                let xor_key = branch_key[byte] ^ keys[i][byte];
                let split_bit = byte * 8 + (7 - f32::from(xor_key).log2().floor() as usize);
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

    fn calc_min_split_index(
        &self,
        keys: &[&[u8]],
        branch_key: &[u8],
    ) -> BinaryMerkleTreeResult<usize> {
        let mut min_key;
        if let Some(m) = keys.iter().min() {
            min_key = *m;
        } else {
            return Err(Exception::new("No keys to calculate minimum split index"));
        }

        let mut max_key;
        if let Some(m) = keys.iter().max() {
            max_key = *m;
        } else {
            return Err(Exception::new("No keys to calculate minimum split index"));
        }

        if branch_key[..] < *min_key {
            min_key = &branch_key;
        } else if branch_key[..] > *max_key {
            max_key = &branch_key;
        }

        let mut split_bit = min_key.len() * 8;
        for i in 0..min_key.len() {
            if min_key[i] == max_key[i] {
                continue;
            }
            let xor_key = min_key[i] ^ max_key[i];
            split_bit = i * 8 + (7 - f32::from(xor_key).log2().floor() as usize);
            break;
        }
        Ok(split_bit)
    }

    fn insert_leaves(
        &mut self,
        keys: &[&[u8]],
        values: &[&ValueType],
    ) -> BinaryMerkleTreeResult<Vec<[u8; 32]>> {
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
            leaf.set_data(&data_node_location);
            let mut leaf_key = [0; 32];
            leaf_key.copy_from_slice(keys[i]);
            leaf.set_key(leaf_key);

            let mut leaf_hasher = HasherType::new(32);
            leaf_hasher.update(b"l");
            leaf_hasher.update(keys[i]);
            leaf_hasher.update(&leaf.get_data());
            let leaf_node_location = leaf_hasher.finalize();

            let mut leaf_node = NodeType::new(NodeVariant::Leaf(leaf));
            leaf_node.set_references(1);

            if let Some(n) = self.db.get_node(&data_node_location)? {
                let references = n.get_references() + 1;
                data_node.set_references(references);
            }

            if let Some(n) = self.db.get_node(&leaf_node_location)? {
                let references = n.get_references() + 1;
                leaf_node.set_references(references);
            }

            self.db.insert(&data_node_location, &data_node)?;
            self.db.insert(&leaf_node_location, &leaf_node)?;

            let mut location = [0; 32];
            location.copy_from_slice(&leaf_node_location);
            nodes.push(location);
        }
        Ok(nodes)
    }

    fn create_tree(&mut self, mut tree_refs: Vec<TreeRef>) -> BinaryMerkleTreeResult<[u8; 32]> {
        if tree_refs.is_empty() {
            return Err(Exception::new("Tried to create a tree with no tree refs"));
        }

        if tree_refs.len() == 1 {
            self.db.batch_write()?;
            let node = tree_refs.remove(0);
            if let Ok(v) = Rc::try_unwrap(node.location) {
                return Ok(v);
            } else {
                return Err(Exception::new("Failed to unwrap tree root"));
            }
        }

        tree_refs.sort();

        let tree_rcs = tree_refs
            .into_iter()
            .map(|x| Rc::new(RefCell::new(TreeRefWrapper::new(Rc::new(RefCell::new(x))))))
            .collect::<Vec<_>>();

        let mut tree_ref_queue = BinaryHeap::with_capacity(tree_rcs.len() - 1);
        let keylen = RefCell::borrow(&tree_rcs[0]).get_tree_ref_key().len();
        for i in 0..tree_rcs.len() - 1 {
            let left_key = &RefCell::borrow(&tree_rcs[i]).get_tree_ref_key();
            let right_key = &RefCell::borrow(&tree_rcs[i + 1]).get_tree_ref_key();

            for j in 0..keylen {
                if j == keylen - 1 && left_key[j] == right_key[j] {
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
                let xor_key = left_key[j] ^ right_key[j];
                let split_bit =
                    (j * 8) as usize + (7 - (f32::from(xor_key).log2().floor()) as usize);

                tree_ref_queue.push((
                    split_bit,
                    Rc::clone(&tree_rcs[i]),
                    Rc::clone(&tree_rcs[i + 1]),
                ));
                break;
            }
        }

        drop(tree_rcs);

        while !tree_ref_queue.is_empty() {
            let item = tree_ref_queue.pop().expect("Tree ref queue is empty");
            let split_index = item.0;

            let mut branch = BranchType::new();
            let branch_node_location;
            let count;

            let tree_ref_wrapper = item.1;
            let next_tree_ref_wrapper = item.2;

            tree_ref_wrapper.borrow_mut().update_reference();
            next_tree_ref_wrapper.borrow_mut().update_reference();

            let tree_ref_key = tree_ref_wrapper.borrow().get_tree_ref_key();
            let tree_ref_location = tree_ref_wrapper.borrow().get_tree_ref_location();
            let tree_ref_count = tree_ref_wrapper.borrow().get_tree_ref_count();

            let next_tree_ref_location = next_tree_ref_wrapper.borrow().get_tree_ref_location();
            let next_tree_ref_count = next_tree_ref_wrapper.borrow().get_tree_ref_count();

            {
                let mut branch_hasher = HasherType::new(32);
                branch_hasher.update(b"b");
                branch_hasher.update(&tree_ref_location[..]);
                branch_hasher.update(&next_tree_ref_location[..]);
                let branch_node_location_vec = branch_hasher.finalize();
                let mut branch_node_location_arr = [0u8; 32];
                branch_node_location_arr.copy_from_slice(&branch_node_location_vec);
                branch_node_location = Rc::new(branch_node_location_arr);

                count = tree_ref_count + next_tree_ref_count;
                let mut branch_zero = [0; 32];
                branch_zero.copy_from_slice(&tree_ref_location[..]);

                let mut branch_one = [0; 32];
                branch_one.copy_from_slice(&next_tree_ref_location[..]);

                let mut branch_key = [0; 32];
                branch_key.copy_from_slice(&tree_ref_key[..]);

                branch.set_zero(branch_zero);
                branch.set_one(branch_one);
                branch.set_count(count);
                branch.set_split_index(split_index as u32);
                branch.set_key(branch_key);
            }

            let mut branch_node = NodeType::new(NodeVariant::Branch(branch));
            branch_node.set_references(1);

            self.db.insert(&branch_node_location[..], &branch_node)?;

            next_tree_ref_wrapper
                .borrow_mut()
                .set_tree_ref_key(Rc::clone(&tree_ref_key));
            next_tree_ref_wrapper
                .borrow_mut()
                .set_tree_ref_location(Rc::clone(&branch_node_location));
            next_tree_ref_wrapper.borrow_mut().set_tree_ref_count(count);

            tree_ref_wrapper
                .borrow_mut()
                .set_reference(next_tree_ref_wrapper);

            if tree_ref_queue.is_empty() {
                self.db.batch_write()?;
                let root = branch_node_location;
                match Rc::try_unwrap(root) {
                    Ok(v) => return Ok(v),
                    Err(v) => return Ok(*v),
                }
            }
        }
        unreachable!();
    }

    /// Remove all items with less than 1 reference under the given root.
    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        if root_hash.len() != 32 {
            return Err(Exception::new("root_hash must be 32 bytes long"));
        }

        let mut nodes = VecDeque::with_capacity(128);
        nodes.push_front(root_hash.to_vec());

        while !nodes.is_empty() {
            let node_location = if let Some(l) = nodes.pop_front() {
                l
            } else {
                return Err(Exception::new("Empty node queue"));
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
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
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
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
        ];
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
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
        ];
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
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
        ];
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
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &zero_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
            &one_key[..],
        ];

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
