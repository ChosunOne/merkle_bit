use std::collections::VecDeque;
use std::path::PathBuf;
use std::error::Error;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::marker::PhantomData;

use traits::{Encode, Exception, Decode, Branch, Data, Hasher, Database, Node, Leaf};

/// A generic Result from an operation involving a MerkleBIT
pub type BinaryMerkleTreeResult<T> = Result<T, Box<Error>>;

/// Contains the distinguishing data from the node
#[derive(Clone)]
pub enum NodeVariant<BranchType, LeafType, DataType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data {
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType)
}

#[derive(Debug, PartialEq)]
enum BranchSplit {
    Zero,
    One
}

struct SplitPairs<'a> {
    zeros: Vec<&'a [u8]>,
    ones: Vec<&'a [u8]>
}

struct TreeCell<'a, NodeType> {
    keys: Vec<&'a [u8]>,
    node: Option<NodeType>,
    depth: usize
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
struct TreeRef {
    key: Vec<u8>,
    location: Vec<u8>,
    count: u64
}

impl Ord for TreeRef {
    fn cmp(&self, other_ref: &TreeRef) -> Ordering {
        return self.key.cmp(&other_ref.key)
    }
}

impl<'a> SplitPairs<'a> {
    pub fn new(zeros: Vec<&'a [u8]>, ones: Vec<&'a [u8]>) -> SplitPairs<'a> {
        SplitPairs {
            zeros,
            ones
        }
    }
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(keys: Vec<&'a [u8]>, node: Option<NodeType>, depth: usize) -> TreeCell<'a, NodeType>
        where BranchType: Branch,
              LeafType: Leaf,
              DataType: Data {
        TreeCell {
            keys,
            node,
            depth
        }
    }
}

impl TreeRef {
    pub fn new(key: Vec<u8>, location: Vec<u8>, count: u64) -> TreeRef {
        TreeRef {
            key,
            location,
            count
        }
    }
}

fn choose_branch(key: &[u8], bit: usize) -> BranchSplit {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    if extracted_bit == 0 {
        return BranchSplit::Zero
    } else {
        return BranchSplit::One
    }
}

fn split_pairs(sorted_pairs: Vec<&[u8]>, bit: usize) ->  SplitPairs {
    if sorted_pairs.len() == 0 {
        return SplitPairs::new(vec![], vec![])
    }

    if let BranchSplit::Zero = choose_branch(sorted_pairs[sorted_pairs.len() - 1], bit) {
        return SplitPairs::new(sorted_pairs[0..sorted_pairs.len()].to_vec(), vec![])
    }

    if let BranchSplit::One = choose_branch(sorted_pairs[0], bit) {
        return SplitPairs::new(vec![], sorted_pairs[0..sorted_pairs.len()].to_vec())
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();

    while max - min > 1 {
        let bisect = (max - min) / 2 + min;
        match choose_branch(sorted_pairs[bisect], bit) {
            BranchSplit::Zero => min = bisect,
            BranchSplit::One =>  max = bisect
        }
    }

    SplitPairs::new(sorted_pairs[0..max].to_vec(), sorted_pairs[max..sorted_pairs.len()].to_vec())
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
    where DatabaseType: Database<NodeType = NodeType>,
          BranchType: Branch,
          LeafType: Leaf,
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
    value: PhantomData<*const ValueType>
}

impl<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, HashResultType, ValueType>
    MerkleBIT<DatabaseType, BranchType, LeafType, DataType, NodeType, HasherType, HashResultType, ValueType>
    where DatabaseType: Database<NodeType = NodeType>,
          BranchType: Branch,
          LeafType: Leaf,
          DataType: Data,
          NodeType: Node<BranchType, LeafType, DataType, ValueType>,
          HasherType: Hasher<HashType = HasherType, HashResultType = HashResultType>,
          HashResultType: AsRef<[u8]> + Clone + Eq + Debug + PartialOrd,
          ValueType: Decode + Encode {
    /// Create a new MerkleBIT from a saved database
    pub fn new(path: PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
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
            value: PhantomData
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
            value: PhantomData
        })
    }

    /// Get items from the MerkleBIT.  Keys must be sorted.  Returns a list of Options which may include the corresponding values.
    pub fn get(&self, root_hash: &HashResultType, keys: Vec<&[u8]>) -> BinaryMerkleTreeResult<Vec<Option<ValueType>>> {
        let root_node;
        if let Some(n) = self.db.get_node(root_hash.as_ref())? {
            root_node = n;
        } else {
            let mut values = Vec::with_capacity(keys.len());
            for _ in 0..keys.len() {
                values.push(None);
            }
            return Ok(values)
        }

        let mut leaf_nodes: VecDeque<Option<LeafType>> = VecDeque::with_capacity(keys.len());

        let mut cell_queue: VecDeque<TreeCell<NodeType>> = VecDeque::with_capacity(2.0_f64.powf(self.depth as f64) as usize);

        let root_cell: TreeCell<NodeType> = TreeCell::new::<BranchType, LeafType, DataType>(keys.clone(), Some(root_node), 0);

        cell_queue.push_front(root_cell);

        while cell_queue.len() > 0 {
            let tree_cell;
            match cell_queue.pop_front() {
                Some(f) => tree_cell = f,
                None => return Err(Box::new(Exception::new("Empty tree_cell queue")))
            }

            if tree_cell.depth > self.depth {
                return Err(Box::new(Exception::new("Depth of merkle tree exceeded")))
            }

            let node;
            match tree_cell.node {
                Some(n) => node = n,
                None => {
                    for _ in 0..tree_cell.keys.len() {
                        leaf_nodes.push_back(None);
                    }
                    continue;
                }
            }

            match node.get_variant()? {
                NodeVariant::Branch(n) => {
                    let split = split_pairs(tree_cell.keys, n.get_split_index() as usize);

                    // If you switch the order of these blocks, the result comes out backwards
                    if let Some(o) = self.db.get_node(n.get_one())? {
                        let one_node = o;
                        if split.ones.len() > 0 {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, Some(one_node), tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    } else {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, None, tree_cell.depth);
                        cell_queue.push_front(new_cell);
                    }

                    if let Some(z) = self.db.get_node(n.get_zero())? {
                        let zero_node = z;
                        if split.zeros.len() > 0 {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.zeros, Some(zero_node), tree_cell.depth + 1);
                            cell_queue.push_front(new_cell);
                        }
                    } else {
                        for _ in 0..split.zeros.len() {
                            leaf_nodes.push_back(None);
                        }
                    }
                },
                NodeVariant::Leaf(n) => {
                    if tree_cell.keys.len() == 0 {
                        return Err(Box::new(Exception::new("No key with which to match the leaf key")))
                    }

                    leaf_nodes.push_back(Some(n));

                    if tree_cell.keys.len() > 1 {
                        for _ in 0..tree_cell.keys.len() - 1 {
                            leaf_nodes.push_back(None);
                        }
                    }
                },
                NodeVariant::Data(_) => {
                    return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }
            }
        }

        let mut values = Vec::with_capacity(leaf_nodes.len());

        for i in 0..leaf_nodes.len() {
            if let Some(ref l) = leaf_nodes[i] {
                if l.get_key() != keys[i] {
                    values.push(None);
                    continue;
                }

                let data = l.get_data();
                let new_node;
                if let Some(e) = self.db.get_node(data)? {
                    new_node = e;
                } else {
                    return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }
                match new_node.get_variant()? {
                    NodeVariant::Data(n) => {
                        values.push(Some(ValueType::decode(n.get_value())?));
                    },
                    _ => {
                        return Err(Box::new(Exception::new("Corrupt merkle tree")))
                    }
                }
            } else {
                values.push(None);
            }
        }

        Ok(values)
    }

    /// Insert items into the MerkleBIT.  Keys must be sorted.  Returns a new root hash for the MerkleBIT.
    pub fn insert(&mut self, previous_root: Option<&HashResultType>, keys: Vec<&[u8]>, values: &[&ValueType]) -> BinaryMerkleTreeResult<Vec<u8>> {
        if keys.len() != values.len() {
            return Err(Box::new(Exception::new("Keys and values have different lengths")))
        }

        let mut nodes: Vec<HashResultType> = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            // Create data node
            let mut data = DataType::new();
            data.set_value(&values[i].encode()?);

            let mut data_hasher = HasherType::new(32);
            data_hasher.update("d".as_bytes());
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
            leaf_hasher.update("l".as_bytes());
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

        let mut tree_refs = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            let tree_ref = TreeRef::new(keys[i].to_vec(), nodes[i].as_ref().to_vec(), 1);
            tree_refs.push(tree_ref);
        }

        if let Some(n) = previous_root {
            // Nodes that form the merkle proof for the new tree
            let mut proof_nodes: Vec<TreeRef> = Vec::new();

            let root_node;
            if let Some(m) = self.db.get_node(n.as_ref())? {
                root_node = m;
            } else {
                return Err(Box::new(Exception::new("Could not find previous root")))
            }

            let mut cell_queue: Vec<TreeCell<NodeType>> = Vec::with_capacity(2.0_f64.powf(self.depth as f64) as usize);
            let root_cell: TreeCell<NodeType> = TreeCell::new::<BranchType, LeafType, DataType>(keys, Some(root_node), 0);
            cell_queue.push(root_cell);

            while cell_queue.len() > 0 {
                let tree_cell = cell_queue.remove(0);

                if tree_cell.depth > self.depth {
                    return Err(Box::new(Exception::new("Depth of merkle tree exceeded")))
                }

                let mut node;
                if let Some(n) = tree_cell.node {
                    node = n;
                } else {
                    continue
                }

                let branch;
                match node.get_variant()? {
                    NodeVariant::Branch(n) => {
                        branch = n
                    },
                    NodeVariant::Leaf(n) => {
                        let leaf = n;
                        let key = leaf.get_key();
                        let data = leaf.get_data();
                        let mut leaf_hasher = HasherType::new(32);
                        leaf_hasher.update("l".as_bytes());
                        leaf_hasher.update(key);
                        leaf_hasher.update(data);
                        let location = leaf_hasher.finalize();

                        let mut skip = false;
                        let mut old = false;

                        // Check if we are updating an existing value
                        for i in 0..tree_refs.len() {
                            let b = &tree_refs[i];
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
                            return Err(Box::new(Exception::new("Corrupt merkle tree")))
                        }

                        if old {
                            continue;
                        }

                        let tree_ref = TreeRef::new(key.to_vec(), location.as_ref().to_vec(), 1);
                        proof_nodes.push(tree_ref);
                        continue;
                    },
                    NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }

                let mut min_split_index = tree_cell.keys[0].len() * 8;
                let mut branch_hasher = HasherType::new(32);
                branch_hasher.update("b".as_bytes());
                branch_hasher.update(branch.get_zero());
                branch_hasher.update(branch.get_one());
                let location = branch_hasher.finalize();
                let branch_key;
                {
                    branch_key = self.get_proof_key(location.as_ref())?;
                    let mut all_keys = tree_cell.keys.clone();
                    all_keys.push(branch_key.as_ref());
                    for i in 0..all_keys.len() - 1 {
                        for j in 0..all_keys[0].len() * 8 {
                            let left = choose_branch(all_keys[i].as_ref(), j);
                            let right = choose_branch(all_keys[i + 1].as_ref(), j);
                            if left != right {
                                if j < min_split_index {
                                    min_split_index = j;
                                }
                                break;
                            }
                        }
                    }
                }

                let mut descendants = Vec::with_capacity(tree_cell.keys.len());

                if min_split_index < branch.get_split_index() as usize {

                    // Check if any keys from the search need to go down this branch
                    for i in 0..tree_cell.keys.len() {
                        let cell_key = tree_cell.keys[i];
                        let mut descendant = true;
                        for j in min_split_index..branch.get_split_index() as usize {
                            let left = choose_branch(&branch_key, j);
                            let right = choose_branch(cell_key, j);
                            if left != right {
                                descendant = false;
                                break;
                            }
                        }
                        if descendant {
                            descendants.push(cell_key);
                        }
                    }

                    if descendants.len() == 0 {
                        let tree_ref = TreeRef::new(branch_key, location.as_ref().to_vec(), branch.get_count());
                        let refs = node.get_references() + 1;
                        node.set_references(refs);
                        self.db.insert(location.as_ref(), &node)?;
                        proof_nodes.push(tree_ref);
                        continue;
                    }
                } else {
                    for i in 0..tree_cell.keys.len(){
                        descendants.push(&tree_cell.keys[i]);
                    }
                }

                let split = split_pairs(descendants, branch.get_split_index() as usize);
                if let Some(mut o) = self.db.get_node(branch.get_one())? {
                    let mut one_node = o;
                    if split.ones.len() > 0 {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.ones, Some(one_node), tree_cell.depth + 1);
                        cell_queue.insert(0, new_cell);
                    } else {
                        let other_key;
                        if let Some(k) = branch.get_key() {
                            other_key = k.to_vec();
                        } else {
                            other_key = self.get_proof_key(branch.get_one())?;
                        }
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
                if let Some(mut z) = self.db.get_node(branch.get_zero())? {
                    let mut zero_node = z;
                    if split.zeros.len() > 0 {
                        let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(split.zeros, Some(zero_node), tree_cell.depth + 1);
                        cell_queue.insert(0, new_cell);
                    } else {
                        let other_key;
                        if let Some(k) = branch.get_key() {
                            other_key = k.to_vec();
                        } else {
                            other_key = self.get_proof_key(branch.get_zero())?;
                        }
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

            let new_root = self.create_tree(&mut tree_refs)?;
            return Ok(new_root)
        } else {
            // There is no tree, just build one with the keys and values
            let new_root = self.create_tree(&mut tree_refs)?;
            return Ok(new_root)
        }
    }

    fn create_tree(&mut self, tree_refs: &mut Vec<TreeRef>) -> BinaryMerkleTreeResult<Vec<u8>> {
        tree_refs.sort();
        let mut split_indices = Vec::with_capacity(tree_refs.len() - 1);
        for i in 0..tree_refs.len() - 1 {
            for j in 0..tree_refs[i].key.len() * 8 {
                let left_branch = choose_branch(tree_refs[i].key.as_ref(), j);
                let right_branch = choose_branch(tree_refs[i + 1].key.as_ref(), j);
                if left_branch != right_branch{
                    split_indices.push(vec![i, j]);
                    break;
                } else if j == tree_refs[i].key.len() * 8 - 1 {
                    // The keys are the same and don't diverge
                    return Err(Box::new(Exception::new("Attempted to insert item with duplicate keys")))
                }
            }
        }

        split_indices.sort_by(|a, b| {
            a[1].cmp(&b[1]).reverse()
        });

        while tree_refs.len() > 0 {
            if tree_refs.len() == 1 {
                self.db.batch_write()?;
                return Ok(tree_refs.remove(0).location.to_vec())
            }

            let max_tree_ref = split_indices.remove(0);
            let max_index= max_tree_ref[0];

            for i in 0..split_indices.len() {
                if split_indices[i][0] > max_index {
                    split_indices[i][0] -= 1;
                }
            }

            let tree_ref = tree_refs.remove(max_index);

            let next_tree_ref = tree_refs.remove(max_index);
            let mut branch_hasher = HasherType::new(32);
            branch_hasher.update("b".as_bytes());
            branch_hasher.update(tree_ref.location.as_ref());
            branch_hasher.update(next_tree_ref.location.as_ref());
            let branch_node_location = branch_hasher.finalize();

            let mut branch = BranchType::new();
            branch.set_zero(tree_ref.location.as_ref());
            branch.set_one(next_tree_ref.location.as_ref());
            let count = tree_ref.count + next_tree_ref.count;
            branch.set_count(count);
            branch.set_split_index(max_tree_ref[1] as u32);

            let mut branch_node = NodeType::new();
            branch_node.set_branch(branch);
            branch_node.set_references(1);

            self.db.insert(branch_node_location.as_ref(), &branch_node)?;
            let new_tree_ref = TreeRef::new(tree_ref.key, branch_node_location.as_ref().to_vec(), count);
            tree_refs.insert(max_index, new_tree_ref);
        }

        Err(Box::new(Exception::new("Corrupt merkle tree")))
    }

    fn get_proof_key(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<Vec<u8>> {
        let mut found_leaf = false;
        let mut key= vec![];
        let mut child_location = root_hash.to_vec();

        // DFS to find a key
        while !found_leaf {
            if let Some(n) = self.db.get_node(child_location.as_ref())? {
                let node = n;
                match node.get_variant()? {
                    NodeVariant::Branch(m) => {
                        child_location = m.get_zero().to_owned()},
                    NodeVariant::Leaf(m) => {
                        key = m.get_key().to_vec();
                        found_leaf = true;
                    },
                    NodeVariant::Data(_) => return Err(Box::new(Exception::new("Corrupt merkle tree")))
                }
            } else {
                return Err(Box::new(Exception::new("Corrupt merkle tree")))
            }
        }
        Ok(key)
    }

    /// Remove all items with less than 1 reference under the given root.
    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        let mut nodes = Vec::with_capacity(128);
        nodes.push(root_hash.to_vec());

        while nodes.len() > 0 {
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
                },
                NodeVariant::Leaf(l) => {
                    if refs == 0 {
                        let data = l.get_data();
                        nodes.push(data.to_vec());
                        self.db.remove(&node_location)?;
                        continue;
                    }
                },
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
    use super::*;

    use blake2_rfc::blake2b::{blake2b};
    use std::collections::HashMap;
    use rand::{Rng, SeedableRng, StdRng};

    fn hash(data: &[u8], size: usize) -> Vec<u8> {
        let hash_data = blake2b(size, &[], data);
        let mut hash_vec = vec![0; size];
        hash_vec.clone_from_slice(&hash_data.as_bytes()[..]);
        hash_vec
    }

    struct MockDB {
        map: HashMap<Vec<u8>, ProtoMerkleNode>,
        pending_inserts: Vec<(Vec<u8>, ProtoMerkleNode)>
    }

    impl MockDB {
        pub fn new(map: HashMap<Vec<u8>, ProtoMerkleNode>) -> MockDB {
            MockDB {
                map,
                pending_inserts: Vec::with_capacity(64)
            }
        }
    }

    impl Database for MockDB {
        type NodeType = ProtoMerkleNode;
        type EntryType = (Vec<u8>, Self::NodeType);

        fn open(_path: PathBuf) -> Result<MockDB, Box<Error>> {
            Ok(MockDB::new(HashMap::new()))
        }

        fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
            if let Some(m) = self.map.get(key) {
                let node = m.clone();
                return Ok(Some(node))
            } else {
                return Ok(None)
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
            while self.pending_inserts.len() > 0 {
                let entry = self.pending_inserts.remove(0);
                self.map.insert(entry.0, entry.1);
            }
            Ok(())
        }
    }

    #[derive(Clone)]
    struct ProtoBranch {
        count: u64,
        zero: Vec<u8>,
        one: Vec<u8>,
        split_index: u32
    }

    impl ProtoBranch {
        fn new() -> ProtoBranch {
            ProtoBranch {
                count: 0,
                zero: vec![],
                one: vec![],
                split_index: 0
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
    }

    #[derive(Clone)]
    struct ProtoLeaf {
        key: Vec<u8>,
        data: Vec<u8>
    }

    impl ProtoLeaf {
        fn new() -> ProtoLeaf {
            ProtoLeaf {
                key: vec![],
                data: vec![]
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

    #[derive(Clone)]
    struct ProtoData {
        value: Vec<u8>
    }

    impl ProtoData {
        fn new() -> ProtoData {
            ProtoData {
                value: vec![]
            }
        }

        fn get_value(&self) -> &[u8] {
            self.value.as_ref()
        }

        fn set_value(&mut self, value: Vec<u8>) {
            self.value = value;
        }
    }

    #[derive(Clone)]
    struct ProtoMerkleNode {
        references: u64,
        node: Option<NodeVariant<ProtoBranch, ProtoLeaf, ProtoData>>
    }

    impl ProtoMerkleNode {
        fn new() -> ProtoMerkleNode {
            ProtoMerkleNode {
                references: 0,
                node: None
            }
        }

        fn get_references(&self) -> u64 {
            self.references
        }

        fn set_references(&mut self, references: u64) {
            self.references = references;
        }

        fn set_branch(&mut self, branch: ProtoBranch) {
            self.node = Some(NodeVariant::Branch(branch));
        }

        fn set_leaf(&mut self, leaf: ProtoLeaf) {
            self.node = Some(NodeVariant::Leaf(leaf));
        }

        fn set_data(&mut self, data: ProtoData) {
            self.node = Some(NodeVariant::Data(data));
        }

    }

    impl Branch for ProtoBranch {
        fn new() -> ProtoBranch {
            ProtoBranch::new()
        }
        fn get_count(&self) -> u64 {
            ProtoBranch::get_count(self)
        }
        fn get_zero(&self) -> &[u8] {
            ProtoBranch::get_zero(self)
        }
        fn get_one(&self) -> &[u8] {
            ProtoBranch::get_one(self)
        }
        fn get_split_index(&self) -> u32 {
            ProtoBranch::get_split_index(self)
        }
        fn get_key(&self) -> Option<&[u8]> { None }
        fn set_count(&mut self, count: u64) {
            ProtoBranch::set_count(self, count);
        }
        fn set_zero(&mut self, zero: &[u8]) {
            ProtoBranch::set_zero(self, zero.to_vec());
        }
        fn set_one(&mut self, one: &[u8]) {
            ProtoBranch::set_one(self, one.to_vec());
        }
        fn set_split_index(&mut self, index: u32) {
            ProtoBranch::set_split_index(self, index);
        }
        fn set_key(&mut self, _key: &[u8]) { }
    }

    impl Leaf for ProtoLeaf {
        fn new() -> ProtoLeaf {
            ProtoLeaf::new()
        }
        fn get_key(&self) -> &[u8] {
            ProtoLeaf::get_key(self)
        }
        fn get_data(&self) -> &[u8] {
            ProtoLeaf::get_data(self)
        }
        fn set_key(&mut self, key: &[u8]) {
            ProtoLeaf::set_key(self, key.to_vec());
        }
        fn set_data(&mut self, data: &[u8]) {
            ProtoLeaf::set_data(self, data.to_vec());
        }
    }

    impl Data for ProtoData {
        fn new() -> ProtoData {
            ProtoData::new()
        }
        fn get_value(&self) -> &[u8] {
            ProtoData::get_value(self)
        }
        fn set_value(&mut self, value: &[u8]) {
            ProtoData::set_value(self, value.to_vec());
        }
    }

    impl Node<ProtoBranch, ProtoLeaf, ProtoData, Vec<u8>> for ProtoMerkleNode {
        fn new() -> ProtoMerkleNode {
            ProtoMerkleNode::new()
        }
        fn get_references(&self) -> u64 {
            ProtoMerkleNode::get_references(self)
        }
        fn get_variant(&self)
                       -> BinaryMerkleTreeResult<NodeVariant<ProtoBranch, ProtoLeaf, ProtoData>>
            where ProtoBranch: Branch,
                  ProtoLeaf: Leaf,
                  ProtoData: Data, {
            match self.node {
                Some(ref node_type) => {
                    match node_type {
                        NodeVariant::Branch(branch) => return Ok(NodeVariant::Branch(branch.clone())),
                        NodeVariant::Data(data) => return Ok(NodeVariant::Data(data.clone())),
                        NodeVariant::Leaf(leaf) => return Ok(NodeVariant::Leaf(leaf.clone()))
                    }
                },
                None => return Err(Box::new(Exception::new("Failed to distinguish node type")))
            }
        }

        fn set_references(&mut self, references: u64) {
            ProtoMerkleNode::set_references(self, references);
        }
        fn set_branch(&mut self, branch: ProtoBranch) {
            ProtoMerkleNode::set_branch(self, branch);
        }
        fn set_leaf(&mut self, leaf: ProtoLeaf) {
            ProtoMerkleNode::set_leaf(self, leaf);
        }
        fn set_data(&mut self, data: ProtoData) {
            ProtoMerkleNode::set_data(self, data);
        }
    }

    impl Encode for ProtoMerkleNode {
        fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
            Ok(vec![])
        }
    }

    impl Decode for ProtoMerkleNode {
        fn decode(_buffer: &[u8]) -> Result<ProtoMerkleNode, Box<Error>> {
            let proto = ProtoMerkleNode::new();
            Ok(proto)
        }
    }

    impl Hasher for Vec<u8> {
        type HashType = Vec<u8>;
        type HashResultType = Vec<u8>;
        fn new(size: usize) -> Self::HashType {
            Vec::with_capacity(size)
        }
        fn update(&mut self, data: &[u8]) {
            for i in 0..data.len() {
                self.push(data[i]);
            }
        }
        fn finalize(self) -> Self::HashResultType {
            self
        }
    }

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
        let zero_key: Vec<u8> = vec![0x00];
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..]
        ];
        let keys = key_vec;

        let result = split_pairs(keys, 0);
        assert_eq!(result.zeros.len(), 10);
        assert_eq!(result.ones.len(), 0);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() {
        let one_key = vec![0xFF];
        let keys = vec![
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(keys, 0);
        assert_eq!(result.zeros.len(), 0);
        assert_eq!(result.ones.len(), 10);
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_even_length_sorted_list_of_pairs() {
        let zero_key = vec![0x00];
        let one_key = vec![0xFF];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(keys, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_zeros() {
        let zero_key = vec![0x00];
        let one_key = vec![0xFF];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let result = split_pairs(keys, 0);
        assert_eq!(result.zeros.len(), 6);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_ones() {
        let zero_key = vec![0x00];
        let one_key = vec![0xFF];
        let keys = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];

        let result = split_pairs(keys, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 6);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_gets_an_item_out_of_a_simple_tree() {
        let mut db = MockDB::new(HashMap::new());
        let key = vec![0xAA];
        let proto_data_node_key = insert_data_node(&mut db, vec![0xFF]);
        let proto_root_node_key = insert_leaf_node(&mut db, key.clone(), proto_data_node_key.clone());

        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let result = bmt.get(&proto_root_node_key, vec![&key[..]]).unwrap();
        assert_eq!(result, vec![Some(vec![0xFFu8])]);
    }

    #[test]
    fn it_fails_to_get_from_empty_tree() {
        let db = MockDB::new(HashMap::new());

        let key = vec![0x00];
        let root_key = vec![0x01];

        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let items = bmt.get(&root_key, vec![&key[..]]).unwrap();
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item() {
        let mut db = MockDB::new(HashMap::new());

        let key = vec![0xAA];

        let data_node_key = insert_data_node(&mut db, vec![0xFF]);
        let leaf_node_key = insert_leaf_node(&mut db, key.clone(), data_node_key.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        let nonexistent_key = vec![0xAB];
        let items = bmt.get(&leaf_node_key, vec![&nonexistent_key[..]]).unwrap();
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_small_balanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(8);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(8);
        for i in 0..8 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = Vec::with_capacity(8);
        for i in 0..8 {
            let value = &keys[i][..];
            get_keys.push(value);
        }
        let root_hash = build_tree(&mut db, 8,  keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_small_unbalanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(7);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(7);
        for i in 0..7 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = Vec::with_capacity(7);
        for i in 0..7 {
            let value = &keys[i][..];
            get_keys.push(value);
        }
        let root_hash = build_tree(&mut db, 7,  keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();


        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_medium_balanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let num_leaves = 256;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let value = &keys[i][..];
            get_keys.push(value);
        }

        let root_hash = build_tree(&mut db, num_leaves, keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 8).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_medium_unbalanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let num_leaves = 255;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let value = &keys[i][..];
            get_keys.push(value);
        }

        let root_hash = build_tree(&mut db, num_leaves,  keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 8).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_large_balanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let num_leaves = 65_536;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let value = &keys[i][..];
            get_keys.push(value);
        }

        let root_hash = build_tree(&mut db, num_leaves, keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_gets_items_from_a_large_unbalanced_tree() {
        let mut db = MockDB::new(HashMap::new());
        let num_leaves = 65_535;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let value = &keys[i][..];
            get_keys.push(value);
        }

        let root_hash = build_tree(&mut db, num_leaves, keys.clone(), values.clone());
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();

        let items = bmt.get(&root_hash, get_keys).unwrap();
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_handles_a_branch_with_one_child() {
        let mut db = MockDB::new(HashMap::new());
        let data = insert_data_node(&mut db, vec![0xFF]);
        let leaf = insert_leaf_node(&mut db, vec![0x00], data);
        let branch = insert_branch_node(&mut db, Some(leaf), None, 0);
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 4).unwrap();

        let zero_key = vec![0x00];
        let one_key = vec![0xFF];
        let items = bmt.get(&branch, vec![&zero_key[..], &one_key[..]]).unwrap();
        assert_eq!(items, vec![Some(vec![0xFF]), None]);
    }

    #[test]
    fn it_handles_a_branch_with_no_children() {
        let mut db = MockDB::new(HashMap::new());
        let branch = insert_branch_node(&mut db, None, None, 0);
        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 4).unwrap();

        let zero_key = vec![0x00];
        let one_key = vec![0xFF];
        let items = bmt.get(&branch, vec![&zero_key[..], &one_key[..]]).unwrap();
        assert_eq!(items, vec![None, None]);
    }

    #[test]
    fn it_gets_items_from_a_complex_tree() {
        // Tree description
        // Node (Letter)
        // Key (Number)
        // Value (Number)
        //
        // A     B      C      D     E     F     G     H     I     J     K     L     M     N     O     P
        // 0x00  0x40, 0x41, 0x60, 0x68, 0x70, 0x71, 0x72, 0x80, 0xC0, 0xC1, 0xE0, 0xE1, 0xE2, 0xF0, 0xF8
        // None, None, None, 0x01, 0x02, None, None, None, 0x03, None, None, None, None, None, 0x04, None

        let mut db = MockDB::new(HashMap::new());
        let data_d = insert_data_node(&mut db, vec![0x01]);
        let leaf_d = insert_leaf_node(&mut db, vec![0x60], data_d);
        let data_e = insert_data_node(&mut db, vec![0x02]);
        let leaf_e = insert_leaf_node(&mut db, vec![0x68], data_e);
        let data_i = insert_data_node(&mut db, vec![0x03]);
        let leaf_i = insert_leaf_node(&mut db, vec![0x80], data_i);
        let data_o = insert_data_node(&mut db, vec![0x04]);
        let leaf_o = insert_leaf_node(&mut db, vec![0xF0], data_o);

        let branch_de = insert_branch_node(&mut db, Some(leaf_d), Some(leaf_e), 4);
        let branch_de_fgh = insert_branch_node(&mut db, Some(branch_de), None, 3);
        let branch_bc_defgh = insert_branch_node(&mut db, None, Some(branch_de_fgh), 2);
        let branch_a_bcdefgh = insert_branch_node(&mut db, None, Some(branch_bc_defgh), 1);

        let branch_op = insert_branch_node(&mut db, Some(leaf_o), None, 4);
        let branch_lmn_op = insert_branch_node(&mut db, None, Some(branch_op), 3);
        let branch_jk_lmnop = insert_branch_node(&mut db, None, Some(branch_lmn_op), 2);
        let branch_i_jklmnop = insert_branch_node(&mut db, Some(leaf_i), Some(branch_jk_lmnop), 1);

        let root_node = insert_branch_node(&mut db, Some(branch_a_bcdefgh), Some(branch_i_jklmnop), 0);

        let key_a = vec![0x00]; // 0000_0000
        let key_b = vec![0x40]; // 0100_0000
        let key_c = vec![0x41]; // 0100_0001
        let key_d = vec![0x60]; // 0110_0000
        let key_e = vec![0x68]; // 0110_1000
        let key_f = vec![0x70]; // 0111_0000
        let key_g = vec![0x71]; // 0111_0001
        let key_h = vec![0x72]; // 0111_0010
        let key_i = vec![0x80]; // 1000_0000
        let key_j = vec![0xC0]; // 1100_0000
        let key_k = vec![0xC1]; // 1100_0001
        let key_l = vec![0xE0]; // 1110_0000
        let key_m = vec![0xE1]; // 1110_0001
        let key_n = vec![0xE2]; // 1110_0010
        let key_o = vec![0xF0]; // 1111_0000
        let key_p = vec![0xF8]; // 1111_1000

        let keys = vec![
            &key_a[..], &key_b[..], &key_c[..], &key_d[..],
            &key_e[..], &key_f[..], &key_g[..], &key_h[..],
            &key_i[..], &key_j[..], &key_k[..], &key_l[..],
            &key_m[..], &key_n[..], &key_o[..], &key_p[..]];

        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 5).unwrap();
        let items = bmt.get(&root_node, keys).unwrap();
        let expected_items = vec![
            None, None, None, Some(vec![0x01]),
            Some(vec![0x02]), None, None, None,
            Some(vec![0x03]), None, None, None,
            None, None, Some(vec![0x04]), None];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_returns_the_same_number_of_values_as_keys() {
        let mut db = MockDB::new(HashMap::new());
        let data = insert_data_node(&mut db, vec![0xFF]);
        let leaf = insert_leaf_node(&mut db, vec![0x00], data);
        let branch = insert_branch_node(&mut db, Some(leaf), None, 0);

        let mut keys = Vec::with_capacity(256);
        for i in 0..256 {
            keys.push(vec![i as u8]);
        }

        let mut get_keys = vec![];
        for i in 0..256 {
            let value = &keys[i];
            get_keys.push(&value[..]);
        }

        let bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let items = bmt.get(&branch, get_keys).unwrap();
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
    }

    #[test]
    fn it_inserts_a_leaf_node_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let key = vec![0xAAu8];
        let data = vec![0xBBu8];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let new_root_hash = bmt.insert(None, vec![&key[..]], &[data.as_ref()]).unwrap();
        let items = bmt.get(&new_root_hash, vec![&key[..]]).unwrap();
        let expected_items = vec![Some(vec![0xBBu8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let key_values = vec![
            vec![0x00u8],
            vec![0x01u8]
        ];
        let keys = vec![key_values[0].as_ref(), key_values[1].as_ref()];
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let data = &[data_values[0].as_ref(), data_values[1].as_ref()];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree_with_first_bit_split() {
        let db = MockDB::new(HashMap::new());
        let key_values = vec![
            vec![0x00u8],
            vec![0x80u8]
        ];
        let keys = vec![key_values[0].as_ref(), key_values[1].as_ref()];
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let data = &[data_values[0].as_ref(), data_values[1].as_ref()];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let key_values = vec![
            vec![0xAAu8],  // 1010_1010
            vec![0xBBu8],  // 1011_1011
            vec![0xCCu8]]; // 1100_1100
        let keys = vec![key_values[0].as_ref(), key_values[1].as_ref(), key_values[2].as_ref()];
        let data_values = vec![vec![0xDDu8], vec![0xEEu8], vec![0xFFu8]];
        let data = &[data_values[0].as_ref(), data_values[1].as_ref(), data_values[2].as_ref()];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        let expected_items = vec![Some(vec![0xDDu8]), Some(vec![0xEEu8]), Some(vec![0xFFu8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_small_even_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xAAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(32, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_small_odd_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(31, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_medium_even_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_medium_odd_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(255, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_large_even_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(4096, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_large_odd_amount_of_nodes_into_empty_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(4095, &mut rng);

        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let expected_items = prepare.2;

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_a_leaf_node_into_a_tree_with_one_item() {
        let db = MockDB::new(HashMap::new());
        let first_key = vec![0xAAu8];
        let first_data = vec![0xBBu8];

        let second_key = vec![0xCCu8];
        let second_data = vec![0xDDu8];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let new_root_hash = bmt.insert(None, vec![first_key.as_ref()], &[first_data.as_ref()]).unwrap();
        let second_root_hash = bmt.insert(Some(&new_root_hash), vec![second_key.as_ref()], &[second_data.as_ref()]).unwrap();

        let items = bmt.get(&second_root_hash, vec![first_key.as_ref(), second_key.as_ref()]).unwrap();
        let expected_items = vec![Some(vec![0xBBu8]), Some(vec![0xDDu8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_tree_with_existing_items() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xCAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare_initial = prepare_inserts(4096, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = vec![];
        let initial_data_values = prepare_initial.1;
        let mut initial_data = vec![];
        for i in 0..initial_data_values.len() {
            initial_keys.push(initial_key_values[i].as_ref());
            initial_data.push(initial_data_values[i].as_ref());
        }

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let first_root_hash = bmt.insert(None, initial_keys.clone(), &initial_data).unwrap();

        let prepare_added = prepare_inserts(4096, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = vec![];
        let added_data_values = prepare_added.1;
        let mut added_data = vec![];

        for i in 0..added_data_values.len() {
            added_keys.push(added_key_values[i].as_ref());
            added_data.push(added_data_values[i].as_ref());
        }

        let second_root_hash = bmt.insert(Some(&first_root_hash), added_keys.clone(), &added_data).unwrap();

        let first_items = bmt.get(&first_root_hash, initial_keys.clone()).unwrap();
        let second_items = bmt.get(&second_root_hash, added_keys.clone()).unwrap();

        let expected_initial_items = prepare_initial.2;
        let expected_added_items = prepare_added.2;

        assert_eq!(first_items, expected_initial_items);
        assert_eq!(second_items, expected_added_items);
    }

    #[test]
    fn it_updates_an_existing_entry() {
        let db = MockDB::new(HashMap::new());
        let key = vec![0xAAu8];
        let first_value = vec![0xBBu8];
        let second_value = vec![0xCCu8];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 3).unwrap();
        let first_root_hash = bmt.insert(None, vec![key.as_ref()], &[first_value.as_ref()]).unwrap();
        let second_root_hash = bmt.insert(Some(&first_root_hash), vec![key.as_ref()], &[second_value.as_ref()]).unwrap();

        let first_item = bmt.get(&first_root_hash, vec![key.as_ref()]).unwrap();
        let expected_first_item = vec![Some(first_value.clone())];

        let second_item = bmt.get(&second_root_hash, vec![key.as_ref()]).unwrap();
        let expected_second_item = vec![Some(second_value.clone())];

        assert_eq!(first_item, expected_first_item);
        assert_eq!(second_item, expected_second_item);
    }

    #[test]
    fn it_updates_multiple_existing_entries() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xEEu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare_initial = prepare_inserts(4096, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = vec![];
        let initial_data_values = prepare_initial.1;
        let mut initial_data = vec![];
        for i in 0..initial_key_values.len() {
            initial_keys.push(initial_key_values[i].as_ref());
            initial_data.push(initial_data_values[i].as_ref());
        }

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

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let first_root_hash = bmt.insert(None, initial_keys.clone(), &initial_data).unwrap();
        let second_root_hash = bmt.insert(Some(&first_root_hash), initial_keys.clone(), &updated_data).unwrap();

        let initial_items = bmt.get(&first_root_hash, initial_keys.clone()).unwrap();
        let updated_items = bmt.get(&second_root_hash, initial_keys.clone()).unwrap();

        let expected_initial_items = prepare_initial.2;
        assert_eq!(initial_items, expected_initial_items);
        assert_eq!(updated_items, expected_updated_data_values);
    }

    #[test]
    fn it_does_not_panic_when_removing_a_nonexistent_node() {
        let db = MockDB::new(HashMap::new());

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let missing_root_hash = vec![0x00u8];
        bmt.remove(&missing_root_hash).unwrap();
    }

    #[test]
    fn it_removes_a_node() {
        let db = MockDB::new(HashMap::new());
        let key = vec![0x00];
        let data = vec![0x01];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let root_hash = bmt.insert(None, vec![key.as_ref()], &[data.as_ref()]).unwrap();

        let inserted_data = bmt.get(&root_hash, vec![key.as_ref()]).unwrap();
        let expected_inserted_data = vec![Some(vec![0x01u8])];
        assert_eq!(inserted_data, expected_inserted_data);

        bmt.remove(&root_hash).unwrap();

        let retrieved_values = bmt.get(&root_hash, vec![key.as_ref()]).unwrap();
        let expected_retrieved_values = vec![None];
        assert_eq!(retrieved_values, expected_retrieved_values);
    }

    #[test]
    fn it_removes_an_entire_tree() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(4096, &mut rng);

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let key_values = prepare.0;
        let data_values = prepare.1;
        let mut keys = vec![];
        let mut data = vec![];
        for i in 0..key_values.len() {
            keys.push(key_values[i].as_ref());
            data.push(data_values[i].as_ref());
        }
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
        let expected_inserted_items = prepare.2;
        let inserted_items = bmt.get(&root_hash, keys.clone()).unwrap();
        assert_eq!(inserted_items, expected_inserted_items);

        bmt.remove(&root_hash).unwrap();
        let removed_items = bmt.get(&root_hash, keys.clone()).unwrap();
        let mut expected_removed_items = vec![];
        for _ in 0..keys.len() {
            expected_removed_items.push(None);
        }
        assert_eq!(removed_items, expected_removed_items);
    }

    #[test]
    fn it_removes_an_old_root() {
        let db = MockDB::new(HashMap::new());

        let first_key = vec![0x00u8];
        let first_data = vec![0x01u8];

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let first_root_hash = bmt.insert(None, vec![first_key.as_ref()], &[first_data.as_ref()]).unwrap();

        let second_key = vec![0x02u8];
        let second_data = vec![0x03u8];

        let second_root_hash = bmt.insert(Some(&first_root_hash), vec![second_key.as_ref()], &[second_data.as_ref()]).unwrap();
        bmt.remove(&first_root_hash).unwrap();

        let retrieved_items = bmt.get(&second_root_hash, vec![first_key.as_ref(), second_key.as_ref()]).unwrap();
        let expected_retrieved_items = vec![Some(vec![0x01u8]), Some(vec![0x03u8])];
        assert_eq!(retrieved_items, expected_retrieved_items);
    }

    #[test]
    fn it_removes_a_small_old_tree() {
        let db = MockDB::new(HashMap::new());

        let first_key = vec![0x00u8];
        let second_key = vec![0x01u8];
        let third_key = vec![0x02u8];
        let fourth_key = vec![0x03u8];

        let first_data = vec![0x04u8];
        let second_data = vec![0x05u8];
        let third_data = vec![0x06u8];
        let fourth_data = vec![0x07u8];

        let first_keys = vec![first_key.as_ref(), second_key.as_ref()];
        let first_entries = &[first_data.as_ref(), second_data.as_ref()];
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let first_root_hash = bmt.insert(None, first_keys, first_entries).unwrap();

        let second_keys = vec![third_key.as_ref(), fourth_key.as_ref()];
        let second_entries = &[third_data.as_ref(), fourth_data.as_ref()];
        let second_root_hash = bmt.insert(Some(&first_root_hash), second_keys, second_entries).unwrap();
        bmt.remove(&first_root_hash).unwrap();

        let items = bmt.get(&second_root_hash, vec![first_key.as_ref(), second_key.as_ref(), third_key.as_ref(), fourth_key.as_ref()]).unwrap();
        let expected_items = vec![Some(first_data.clone()), Some(second_data.clone()), Some(third_data.clone()), Some(fourth_data.clone())];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_removes_an_old_large_root() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xBAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare_initial = prepare_inserts(16, &mut rng);
        let initial_key_values = prepare_initial.0;
        let initial_data_values = prepare_initial.1;
        let mut initial_keys = vec![];
        let mut initial_data = vec![];

        for i in 0..initial_key_values.len() {
            initial_keys.push(initial_key_values[i].as_ref());
            initial_data.push(initial_data_values[i].as_ref());
        }

        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();
        let first_root_hash = bmt.insert(None, initial_keys, &initial_data).unwrap();

        let prepare_added = prepare_inserts(16, &mut rng);
        let added_key_values = prepare_added.0;
        let added_data_values = prepare_added.1;
        let mut added_keys = vec![];
        let mut added_data = vec![];

        for i in 0..added_key_values.len() {
            added_keys.push(added_key_values[i].as_ref());
            added_data.push(added_data_values[i].as_ref());
        }

        let second_root_hash = bmt.insert(Some(&first_root_hash), added_keys, &added_data).unwrap();

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

        bmt.remove(&first_root_hash).unwrap();
        let items = bmt.get(&second_root_hash, combined_keys).unwrap();
        assert_eq!(items, combined_expected_items);
    }

    #[test]
    fn it_iterates_over_multiple_inserts_correctly() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xEFu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        iterate_inserts(8, 100, &mut rng, &mut bmt);
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_not_descendants() {
        let db = MockDB::new(HashMap::new());
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        let key_values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x10u8], vec![0x20u8]];
        let mut keys = Vec::with_capacity(key_values.len());
        for i in 0..key_values.len() {
            keys.push(key_values[i].as_ref());
        }
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            data.push(values[i].as_ref());
        }

        let first_root = bmt.insert(None, keys[0..2].to_vec(), &data[0..2]).unwrap();
        let second_root = bmt.insert(Some(&first_root), keys[2..].to_vec(), &data[2..]).unwrap();

        let items = bmt.get(&second_root, keys).unwrap();
        let mut expected_items = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            expected_items.push(Some(values[i].clone()));
        }

        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_descendants() {
        let db = MockDB::new(HashMap::new());
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        let key_values = vec![vec![0x10u8], vec![0x11u8], vec![0x00u8], vec![0x01u8], vec![0x02u8]];
        let mut keys = Vec::with_capacity(key_values.len());
        for i in 0..key_values.len() {
            keys.push(key_values[i].as_ref());
        }
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            data.push(values[i].as_ref());
        }

        let first_root = bmt.insert(None, keys[0..2].to_vec(), &data[0..2]).unwrap();
        let second_root = bmt.insert(Some(&first_root), keys[2..].to_vec(), &data[2..]).unwrap();

        keys.sort();

        let items = bmt.get(&second_root, keys).unwrap();
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8]), Some(vec![0x04u8]), Some(vec![0x00u8]), Some(vec![0x01u8])];
        assert_eq!(items, expected_items);
    }

    #[test]
    fn it_correctly_iterates_removals() {
        let db = MockDB::new(HashMap::new());
        let seed = [0xA8u8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        iterate_removals(8, 100, 1, &mut rng, &mut bmt);
    }

    #[test]
    fn it_correctly_increments_a_leaf_reference_count() {
        let db = MockDB::new(HashMap::new());
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, Vec<u8>, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 160).unwrap();

        let key = vec![0x00u8];
        let data = vec![0x00u8];

        let first_root = bmt.insert(None, vec![key.as_ref()], &[data.as_ref()]).unwrap();
        let second_root = bmt.insert(Some(&first_root), vec![key.as_ref()], &[data.as_ref()]).unwrap();
        bmt.remove(&first_root).unwrap();
        let item = bmt.get(&second_root, vec![key.as_ref()]).unwrap();
        let expected_item = vec![Some(vec![0x00u8])];
        assert_eq!(item, expected_item);
    }

    fn insert_data_node(db: &mut MockDB, value: Vec<u8>) -> Vec<u8> {
        let data_key = hash(&value, 32);

        let mut proto_data_node = ProtoData::new();
        proto_data_node.set_value(value.clone());
        let mut proto_outer_data_node = ProtoMerkleNode::new();
        proto_outer_data_node.set_references(1);
        proto_outer_data_node.set_data(proto_data_node);
        db.insert(&data_key, &proto_outer_data_node).unwrap();
        db.batch_write().unwrap();
        data_key.clone()
    }

    fn insert_leaf_node(db: &mut MockDB, leaf_key: Vec<u8>, data_key: Vec<u8>) -> Vec<u8> {

        let mut proto_leaf_node = ProtoLeaf::new();
        proto_leaf_node.set_data(data_key.clone());
        proto_leaf_node.set_key(leaf_key.clone());

        let mut new_key = leaf_key.clone();
        new_key.append(&mut data_key.clone());
        let leaf_node_key = hash(&new_key, 32);

        let mut proto_outer_leaf_node = ProtoMerkleNode::new();
        proto_outer_leaf_node.set_references(1);
        proto_outer_leaf_node.set_leaf(proto_leaf_node);
        db.insert(&leaf_node_key, &proto_outer_leaf_node).unwrap();
        db.batch_write().unwrap();
        leaf_node_key.clone()
    }

    fn insert_branch_node(db: &mut MockDB, zero_key: Option<Vec<u8>>, one_key: Option<Vec<u8>>, split_index: u32) -> Vec<u8> {
        let mut proto_branch_node = ProtoBranch::new();
        let mut proto_branch_node_key_material;

        if let Some(z) = zero_key {
            proto_branch_node_key_material = z.clone();
            proto_branch_node.set_zero(z.clone());
            proto_branch_node.set_count(1);
            if let Some(o) = one_key {
                proto_branch_node_key_material.append(&mut o.clone());
                proto_branch_node.set_one(o.clone());
                proto_branch_node.set_count(2);
            }
        } else if let Some(o) = one_key {
            proto_branch_node_key_material = o.clone();
            proto_branch_node.set_one(o.clone());
            proto_branch_node.set_count(1);
        } else {
            proto_branch_node_key_material = vec![];
            proto_branch_node.set_count(0);
        }
        proto_branch_node.set_split_index(split_index);
        let proto_branch_node_key = hash(&proto_branch_node_key_material, 32);


        let mut proto_outer_branch_node = ProtoMerkleNode::new();
        proto_outer_branch_node.set_references(1);
        proto_outer_branch_node.set_branch(proto_branch_node);
        db.insert(&proto_branch_node_key, &proto_outer_branch_node).unwrap();
        db.batch_write().unwrap();
        proto_branch_node_key.clone()
    }

    fn build_tree(db: &mut MockDB, num_data_nodes: usize, keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>) -> Vec<u8> {
        if num_data_nodes == 0 {
            return vec![]
        }
        let mut data_node_keys = Vec::with_capacity(num_data_nodes);
        for i in 0..num_data_nodes {
            let value = &values[i];
            data_node_keys.push(insert_data_node(db, value.clone()));
        }

        let depth = (num_data_nodes as f64).log2().ceil() as usize;
        let mut leaf_node_keys = Vec::with_capacity(num_data_nodes);
        for i in 0..data_node_keys.len() {
            let key = data_node_keys[i].clone();

            let mut leaf_key = keys[i].clone();
            leaf_node_keys.push(insert_leaf_node(db, leaf_key, key));
        }

        if leaf_node_keys.len() == 1 {
            return leaf_node_keys[0].clone()
        }

        let mut previous_level = leaf_node_keys;
        for i in (0..depth + 1).rev() {
            let mut branch_node_keys = Vec::with_capacity(previous_level.len() / 2);
            for j in (0..previous_level.len()).step_by(2) {
                if j + 1 > previous_level.len() - 1 {
                    branch_node_keys.push(previous_level[j].clone());
                } else {
                    branch_node_keys.push(insert_branch_node(db, Some(previous_level[j].clone()), Some(previous_level[j + 1].clone()), (i - 1) as u32));
                }
            }
            previous_level = branch_node_keys;
        }
        previous_level[0].clone()
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
                       bmt: &mut MerkleBIT<
                           MockDB,
                           ProtoBranch,
                           ProtoLeaf,
                           ProtoData,
                           ProtoMerkleNode,
                           Vec<u8>,
                           Vec<u8>,
                           Vec<u8>>) -> (Vec<Option<Vec<u8>>>, Vec<Vec<Vec<u8>>>, Vec<Vec<Option<Vec<u8>>>>) {
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

            let mut keys = Vec::with_capacity(key_values.len());
            let mut data = Vec::with_capacity(data_values.len());

            for j in 0..key_values.len() {
                keys.push(key_values[j].as_ref());
                data.push(data_values[j].as_ref());
            }

            let previous_state_root = &state_roots[i].clone();
            let mut previous_root;
            match previous_state_root {
                Some(r) => previous_root = Some(r),
                None => previous_root = None
            }

            let new_root = bmt.insert(previous_root, keys.clone(), &data).unwrap();
            state_roots.push(Some(new_root));

            let retrieved_items = bmt.get(&state_roots[i + 1].clone().unwrap(), keys.clone()).unwrap();
            assert_eq!(retrieved_items, expected_data_values);


            for j in 0..key_groups.len() {
                let mut key_block = Vec::with_capacity(key_groups[j].len());
                for k in 0..key_groups[j].len() {
                    key_block.push(key_groups[j][k].as_ref());
                }
                let items = bmt.get(&state_roots[i + 1].clone().unwrap(), key_block).unwrap();
                assert_eq!(items, data_groups[j]);
            }
        }
        (state_roots, key_groups, data_groups)
    }

    fn iterate_removals(entries_per_insert: usize,
                        iterations: usize,
                        removal_frequency: usize,
                        rng: &mut StdRng,
                        bmt: &mut MerkleBIT<
                            MockDB,
                            ProtoBranch,
                            ProtoLeaf,
                            ProtoData,
                            ProtoMerkleNode,
                            Vec<u8>,
                            Vec<u8>,
                            Vec<u8>>) {
        let inserts =  iterate_inserts(entries_per_insert, iterations, rng, bmt);
        let state_roots = inserts.0;
        let key_groups = inserts.1;
        let data_groups = inserts.2;

        for i in 1..iterations {
            println!("{}", i);
            if i % removal_frequency == 0 {
                let root;
                if let Some(r) = state_roots[i].clone() {
                    root = r.clone();
                } else {
                    panic!("state_roots[{}] is None", i);
                }
                bmt.remove(root.as_ref()).unwrap();
                for j in 0..iterations {
                    let mut keys = Vec::with_capacity(key_groups[i].len());
                    for k in 0..key_groups[i].len() {
                        keys.push(key_groups[i][k].as_ref());
                    }
                    let items = bmt.get(root.as_ref(), keys).unwrap();
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
    }
}