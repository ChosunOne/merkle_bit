#![allow(unused_qualifications)]
#![allow(clippy::std_instead_of_alloc)]
use core::convert::TryFrom;
#[cfg(not(any(feature = "hashbrown")))]
use std::collections::HashMap;

use std::collections::VecDeque;
use std::path::Path;

use crate::Array;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

use crate::prelude::*;
use crate::utils::tree_cell::TreeCell;
use crate::utils::tree_ref::TreeRef;
use crate::utils::tree_utils::*;

/// A generic `Result` from an operation involving a `MerkleBIT`
pub type BinaryMerkleTreeResult<T> = Result<T, MerkleBitError>;

/// A trait collecting all the associated types for the `Merkle-BIT`.
pub trait MerkleTree<const N: usize> {
    /// The type to use for database-like operations.  `Database` must implement the `Database` trait.
    type Database: Database<N, Self::Node>;
    /// The type used for representing branches in the tree. `Branch` must implement the `Branch` trait.
    type Branch: Branch<N>;
    /// The type used for representing leaves in the tree.  `Leaf` must implement the `Leaf` trait.
    type Leaf: Leaf<N>;
    /// The type used for representing data nodes in the tree.  `Data` must implement the `Data` trait.
    type Data: Data;
    ///  The type used for the outer node that can be either a branch, leaf, or data.  `Node` must implement the `Node` trait.
    type Node: Node<N, Branch = Self::Branch, Leaf = Self::Leaf, Data = Self::Data>;
    /// The type of hasher to use for hashing locations on the tree.  `Hasher` must implement the `Hasher` trait.
    type Hasher: Hasher<N>;
    /// The type to return from a get.  `Value` must implement the `Encode` and `Decode` traits.
    type Value: Decode + Encode;
}

/// The `MerkleBIT` struct.
/// # Properties
/// * **db**: The database to store and retrieve values.
/// * **depth**: The maximum permitted depth of the tree.
pub struct MerkleBIT<M: MerkleTree<N>, const N: usize> {
    /// The database to store tree nodes.
    db: M::Database,
    /// The maximum depth of the tree.
    depth: usize,
}

impl<M: MerkleTree<N>, const N: usize> MerkleBIT<M, N> {
    /// Create a new `MerkleBIT` from a saved database
    /// # Errors
    /// `Exception` generated if the `open` fails.
    #[inline]
    pub fn new(path: &Path, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = Database::open(path)?;
        Ok(Self { db, depth })
    }

    /// Create a new `MerkleBIT` from an already opened database
    /// # Errors
    /// None.
    #[inline]
    pub const fn from_db(db: M::Database, depth: usize) -> BinaryMerkleTreeResult<Self> {
        Ok(Self { db, depth })
    }

    /// Get items from the `MerkleBIT`.  Returns a map of `Option`s which may include the corresponding values.
    /// # Errors
    /// `Exception` generated when an invalid state is encountered during tree traversal.
    #[inline]
    pub fn get(
        &self,
        root_hash: &Array<N>,
        keys: &mut [Array<N>],
    ) -> BinaryMerkleTreeResult<HashMap<Array<N>, Option<M::Value>>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut leaf_map = generate_leaf_map(keys);

        keys.sort_unstable();

        let Some(root_node) = self.db.get_node(*root_hash)? else {
            return Ok(leaf_map);
        };

        let mut cell_queue = VecDeque::with_capacity(keys.len());

        let root_cell =
            TreeCell::new::<M::Branch, M::Leaf, M::Data>(*root_hash, keys, root_node, 0);

        cell_queue.push_front(root_cell);

        while let Some(tree_cell) = cell_queue.pop_front() {
            if tree_cell.depth > self.depth {
                return Err(MerkleBitError::DepthExceeded(tree_cell.depth));
            }

            let node = tree_cell.node;

            match node.get_variant() {
                NodeVariant::Branch(branch) => {
                    let (_, zero, one, branch_split_index, branch_key) = branch.decompose();
                    let min_split_index = calc_min_split_index(tree_cell.keys, &branch_key)?;
                    let descendants = check_descendants(
                        tree_cell.keys,
                        branch_split_index,
                        &branch_key,
                        min_split_index,
                    )?;
                    if descendants.is_empty() {
                        continue;
                    }

                    let (zeros, ones) = split_pairs(descendants, branch_split_index)?;

                    self.push_cell_if_node(&mut cell_queue, tree_cell.depth, one, ones)?;
                    self.push_cell_if_node(&mut cell_queue, tree_cell.depth, zero, zeros)?;
                }
                NodeVariant::Leaf(n) => {
                    let d = self
                        .db
                        .get_node(*n.get_data())?
                        .ok_or(CorruptTreeError::NoLeafFromDB)?;
                    let NodeVariant::Data(data) = d.get_variant() else {
                        return Err(CorruptTreeError::NonDataAfterLeaf.into());
                    };
                    let value = M::Value::decode(data.get_value())?;
                    if let Ok(index) = keys.binary_search(n.get_key()) {
                        leaf_map.insert(keys[index], Some(value));
                    }
                }
                NodeVariant::Data(_) => {
                    return Err(CorruptTreeError::DataInTree.into());
                }
            }
        }

        Ok(leaf_map)
    }

    /// Pushes a `TreeCell` to the `cell_queue` if the node exists.
    fn push_cell_if_node<'keys>(
        &self,
        cell_queue: &mut VecDeque<TreeCell<'keys, M::Node, N>>,
        depth: usize,
        location: Array<N>,
        locations: &'keys [Array<N>],
    ) -> BinaryMerkleTreeResult<()> {
        if let Some(node) = self.db.get_node(location)? {
            if !locations.is_empty() {
                let new_cell = TreeCell::new::<M::Branch, M::Leaf, M::Data>(
                    location,
                    locations,
                    node,
                    depth.saturating_add(1),
                );
                cell_queue.push_front(new_cell);
            }
        }
        Ok(())
    }

    /// Insert items into the `MerkleBIT`.  Keys must be sorted.  Returns a new root hash for the `MerkleBIT`.
    /// # Errors
    /// `Exception` generated if an invalid state is encountered during tree traversal.
    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&Array<N>>,
        keys: &mut [Array<N>],
        values: &[M::Value],
    ) -> BinaryMerkleTreeResult<Array<N>> {
        if keys.len() != values.len() {
            return Err(MerkleBitError::KeyValueLengthMismatch((
                keys.len(),
                values.len(),
            )));
        }

        if keys.is_empty() || values.is_empty() {
            return Err(MerkleBitError::EmptyKeysOrValues);
        }

        let mut value_map = HashMap::new();
        for (&key, value) in keys.iter().zip(values.iter()) {
            value_map.insert(key, value);
        }

        keys.sort_unstable();

        let nodes = self.insert_leaves(keys, &value_map)?;

        let mut tree_refs = Vec::with_capacity(keys.len());
        let mut key_map = HashMap::new();
        for (loc, &key) in nodes.into_iter().zip(keys.iter()) {
            key_map.insert(key, loc);
            let tree_ref = TreeRef::new(key, loc, 1, 1);
            tree_refs.push(tree_ref);
        }

        if let Some(root) = previous_root {
            let mut proof_nodes = self.generate_treerefs(root, keys, &key_map)?;
            tree_refs.append(&mut proof_nodes);
        }

        let new_root = self.create_tree(tree_refs)?;
        Ok(new_root)
    }

    /// Traverses the tree and searches for nodes to include in the merkle proof.
    /// # Errors
    /// `Exception` generated when an invalid state is encountered during tree traversal.
    fn generate_treerefs(
        &mut self,
        root: &Array<N>,
        keys: &mut [Array<N>],
        key_map: &HashMap<Array<N>, Array<N>>,
    ) -> BinaryMerkleTreeResult<Vec<TreeRef<N>>> {
        // Nodes that form the merkle proof for the new tree
        let mut proof_nodes = Vec::with_capacity(keys.len());

        let root_node = self.db.get_node(*root)?.ok_or(MerkleBitError::NoRoot)?;

        let mut cell_queue = VecDeque::with_capacity(keys.len());
        let root_cell: TreeCell<M::Node, N> =
            TreeCell::new::<M::Branch, M::Leaf, M::Data>(*root, keys, root_node, 0);
        cell_queue.push_front(root_cell);

        self.traverse_tree(key_map, &mut proof_nodes, &mut cell_queue)?;
        Ok(proof_nodes)
    }

    /// Traverse the tree and append proof nodes
    fn traverse_tree(
        &mut self,
        key_map: &HashMap<Array<N>, Array<N>>,
        proof_nodes: &mut Vec<TreeRef<N>>,
        cell_queue: &mut VecDeque<TreeCell<M::Node, N>>,
    ) -> BinaryMerkleTreeResult<()> {
        while let Some(tree_cell) = cell_queue.pop_front() {
            if tree_cell.depth > self.depth {
                return Err(MerkleBitError::DepthExceeded(tree_cell.depth));
            }

            let node = tree_cell.node;
            let depth = tree_cell.depth;
            let location = tree_cell.location;

            let mut refs = node.get_references();
            let branch = match node.get_variant() {
                NodeVariant::Branch(n) => n,
                NodeVariant::Leaf(n) => {
                    let key = n.get_key();
                    let mut update = false;

                    // Check if we are updating an existing value
                    if let Some(loc) = key_map.get(key) {
                        update = loc == &location;
                        if !update {
                            continue;
                        }
                    }

                    self.insert_leaf(&location)?;

                    if update {
                        continue;
                    }

                    let tree_ref = TreeRef::new(*key, location, 1, 1);
                    proof_nodes.push(tree_ref);
                    continue;
                }
                NodeVariant::Data(_) => {
                    return Err(CorruptTreeError::DataInTree.into());
                }
            };

            let (branch_count, branch_zero, branch_one, branch_split_index, branch_key) =
                branch.decompose();

            let min_split_index = calc_min_split_index(tree_cell.keys, &branch_key)?;

            let mut descendants = tree_cell.keys;

            if min_split_index < branch_split_index {
                descendants = check_descendants(
                    tree_cell.keys,
                    branch_split_index,
                    &branch_key,
                    min_split_index,
                )?;

                if descendants.is_empty() {
                    let mut new_branch = M::Branch::new();
                    new_branch.set_count(branch_count);
                    new_branch.set_zero(branch_zero);
                    new_branch.set_one(branch_one);
                    new_branch.set_split_index(branch_split_index);
                    new_branch.set_key(branch_key);

                    let tree_ref = TreeRef::new(branch_key, tree_cell.location, branch_count, 1);
                    refs = refs.saturating_add(1);
                    let mut new_node = M::Node::new(NodeVariant::Branch(new_branch));
                    new_node.set_references(refs);
                    self.db.insert(tree_ref.location, new_node)?;
                    proof_nodes.push(tree_ref);
                    continue;
                }
            }

            let (zeros, ones) = split_pairs(descendants, branch_split_index)?;
            {
                match self.split_nodes(depth, branch_one, ones)? {
                    SplitNodeType::Ref(tree_ref) => proof_nodes.push(tree_ref),
                    SplitNodeType::Cell(cell) => cell_queue.push_front(cell),
                }
            }
            {
                match self.split_nodes(depth, branch_zero, zeros)? {
                    SplitNodeType::Ref(tree_ref) => proof_nodes.push(tree_ref),
                    SplitNodeType::Cell(cell) => cell_queue.push_front(cell),
                }
            }
        }

        Ok(())
    }

    /// Inserts a leaf into the DB
    fn insert_leaf(&mut self, location: &Array<N>) -> BinaryMerkleTreeResult<()> {
        let mut l = self
            .db
            .get_node(*location)?
            .ok_or(CorruptTreeError::NoLeafFromDB)?;
        let leaf_refs = l.get_references().saturating_add(1);
        l.set_references(leaf_refs);
        self.db.insert(*location, l)?;
        Ok(())
    }

    /// Splits nodes during tree traversal into either zeros or ones, depending on the selected bit
    /// from the index
    /// # Errors
    /// `Exception` generated when an invalid state is encountered during tree traversal.
    fn split_nodes<'node_list>(
        &mut self,
        depth: usize,
        branch: Array<N>,
        node_list: &'node_list [Array<N>],
    ) -> Result<SplitNodeType<'node_list, M::Node, N>, MerkleBitError> {
        let node = self
            .db
            .get_node(branch)?
            .ok_or(CorruptTreeError::NoNodeFromDB)?;
        return if node_list.is_empty() {
            let other_key;
            let count;
            let refs = node.get_references().saturating_add(1);
            let mut new_node;
            match node.get_variant() {
                NodeVariant::Branch(b) => {
                    count = b.get_count();
                    other_key = *b.get_key();
                    new_node = M::Node::new(NodeVariant::Branch(b));
                }
                NodeVariant::Leaf(l) => {
                    count = 1;
                    other_key = *l.get_key();
                    new_node = M::Node::new(NodeVariant::Leaf(l));
                }
                NodeVariant::Data(_) => {
                    return Err(CorruptTreeError::DataInTree.into());
                }
            }
            new_node.set_references(refs);
            self.db.insert(branch, new_node)?;
            let tree_ref = TreeRef::new(other_key, branch, count, 1);
            Ok(SplitNodeType::Ref(tree_ref))
        } else {
            let new_cell = TreeCell::new::<M::Branch, M::Leaf, M::Data>(
                branch,
                node_list,
                node,
                depth.saturating_add(1),
            );
            Ok(SplitNodeType::Cell(new_cell))
        };
    }

    /// Inserts all the new leaves into the database.
    /// Updates reference count if a leaf already exists.
    fn insert_leaves(
        &mut self,
        keys: &[Array<N>],
        values: &HashMap<Array<N>, &M::Value>,
    ) -> BinaryMerkleTreeResult<Vec<Array<N>>> {
        let mut nodes = Vec::with_capacity(keys.len());
        for k in keys.iter() {
            let key = k.as_ref();
            // Create data node
            let mut data = M::Data::new();
            data.set_value(&(values[k].encode()?));

            let mut data_hasher = M::Hasher::new(key.len());
            data_hasher.update(b"d");
            data_hasher.update(key);
            data_hasher.update(data.get_value());
            let data_node_location = data_hasher.finalize();

            let mut data_node = M::Node::new(NodeVariant::Data(data));
            data_node.set_references(1);

            // Create leaf node
            let mut leaf = M::Leaf::new();
            leaf.set_data(data_node_location);
            leaf.set_key(*k);

            let mut leaf_hasher = M::Hasher::new(key.len());
            leaf_hasher.update(b"l");
            leaf_hasher.update(key);
            leaf_hasher.update(leaf.get_data().as_ref());
            let leaf_node_location = leaf_hasher.finalize();

            let mut leaf_node = M::Node::new(NodeVariant::Leaf(leaf));
            leaf_node.set_references(1);

            if let Some(n) = self.db.get_node(data_node_location)? {
                let references = n.get_references().saturating_add(1);
                data_node.set_references(references);
            }

            if let Some(n) = self.db.get_node(leaf_node_location)? {
                let references = n.get_references().saturating_add(1);
                leaf_node.set_references(references);
            }

            self.db.insert(data_node_location, data_node)?;
            self.db.insert(leaf_node_location, leaf_node)?;

            nodes.push(leaf_node_location);
        }
        Ok(nodes)
    }

    /// This function generates the queue of `TreeRef`s and merges the queue together to create a
    /// new tree root.
    /// # Errors
    /// `Exception` generated when `tree_refs` is empty or an invalid state is encountered during
    /// tree traversal
    fn create_tree(&mut self, mut tree_refs: Vec<TreeRef<N>>) -> BinaryMerkleTreeResult<Array<N>> {
        if tree_refs.is_empty() {
            return Err(MerkleBitError::EmptyTreeRefs);
        }

        if tree_refs.len() == 1 {
            self.db.batch_write()?;
            let node = tree_refs.remove(0);
            return Ok(node.location);
        }

        tree_refs.sort();

        let mut tree_ref_queue = HashMap::new();

        let unique_split_bits = generate_tree_ref_queue(&mut tree_refs, &mut tree_ref_queue)?;
        let mut indices = unique_split_bits.into_iter().collect::<Vec<_>>();
        indices.sort_unstable();

        let mut root = None;
        for i in indices.into_iter().rev() {
            let level = tree_ref_queue
                .remove(&i)
                .ok_or(MerkleBitError::EmptyLevel)?;
            root = self.merge_nodes(&mut tree_refs, level)?;
        }
        root.map_or_else(|| Err(MerkleBitError::NoRoot), Ok)
    }

    /// Performs the merging of `TreeRef`s until a single new root is left.
    /// You can visualize the algorithm like the following:  

    /// If two nodes are already adjacent, then create a branch node with the two nodes as children.
    /// After merging, update the right child to be the new node, and the left child to point to it.
    /// ```text
    /// nodes: [A, B, C] -> create branch node D with children A and B, update B to D and A to point to D
    ///        [&D, D, C] -> create branch node E with children D and C, update C to be E and D to point to E
    ///        [&E, &E, E] -> E is the root node, so return E's location
    /// This produces the following tree:
    ///      E
    ///     /\
    ///    D  C
    ///   /\
    ///  A  B  
    /// ```
    /// If the two nodes are not adjacent, find the other node by following the pointer trail.
    fn merge_nodes(
        &mut self,
        tree_refs: &mut [TreeRef<N>],
        level: Vec<(usize, usize, usize)>,
    ) -> BinaryMerkleTreeResult<Option<Array<N>>> {
        #[cfg(feature = "serde")]
        let mut root = Array::default();
        #[cfg(not(any(feature = "serde")))]
        let mut root = [0; N];
        for (split_index, tree_ref_pointer, next_tree_ref_pointer) in level {
            let mut branch = M::Branch::new();

            let tree_ref_key = tree_refs[tree_ref_pointer].key;
            let tree_ref_location = tree_refs[tree_ref_pointer].location;
            let tree_ref_count = tree_refs[tree_ref_pointer].node_count;

            // Find the rightmost edge of the adjacent subtree
            let mut lookahead_count;
            let mut lookahead_tree_ref_pointer: usize;
            {
                let mut count_ = tree_refs[next_tree_ref_pointer].count;

                if count_ > 1 {
                    // Look ahead by the count from our position
                    lookahead_tree_ref_pointer =
                        tree_ref_pointer.saturating_add(usize::try_from(count_)?);
                    lookahead_count = tree_refs[lookahead_tree_ref_pointer].count;
                    while lookahead_count > count_ {
                        count_ = lookahead_count;
                        lookahead_tree_ref_pointer =
                            tree_ref_pointer.saturating_add(usize::try_from(count_)?);
                        lookahead_count = tree_refs[lookahead_tree_ref_pointer].count;
                    }
                } else {
                    lookahead_count = count_;
                    lookahead_tree_ref_pointer = next_tree_ref_pointer;
                }
            }

            let next_tree_ref_location = tree_refs[lookahead_tree_ref_pointer].location;
            let count =
                tree_ref_count.saturating_add(tree_refs[lookahead_tree_ref_pointer].node_count);
            let branch_node_location;
            {
                let mut branch_hasher = M::Hasher::new(root.len());
                branch_hasher.update(b"b");
                branch_hasher.update(&tree_ref_location[..]);
                branch_hasher.update(&next_tree_ref_location[..]);
                branch_node_location = branch_hasher.finalize();

                branch.set_zero(tree_ref_location);
                branch.set_one(next_tree_ref_location);
                branch.set_count(count);
                branch.set_split_index(split_index);
                branch.set_key(tree_ref_key);
            }

            let mut branch_node = M::Node::new(NodeVariant::Branch(branch));
            branch_node.set_references(1);

            self.db.insert(branch_node_location, branch_node)?;

            {
                tree_refs[lookahead_tree_ref_pointer].key = tree_ref_key;
                tree_refs[lookahead_tree_ref_pointer].location = branch_node_location;
                tree_refs[lookahead_tree_ref_pointer].count =
                    lookahead_count.saturating_add(tree_refs[tree_ref_pointer].count);
                tree_refs[lookahead_tree_ref_pointer].node_count = count;
                tree_refs[tree_ref_pointer] = tree_refs[lookahead_tree_ref_pointer];
            }

            root = branch_node_location;
        }
        self.db.batch_write()?;
        Ok(Some(root))
    }

    /// Remove all items with less than 1 reference under the given root.
    /// # Errors
    /// `Exception` generated when an invalid state is encountered during tree traversal.
    #[inline]
    pub fn remove(&mut self, root_hash: &Array<N>) -> BinaryMerkleTreeResult<()> {
        let mut nodes = VecDeque::with_capacity(128);
        nodes.push_front(*root_hash);

        while !nodes.is_empty() {
            let node_location = nodes.pop_front().ok_or(MerkleBitError::NoNodes)?;

            let Some(node) = self.db.get_node(node_location)? else {
                continue;
            };

            let mut refs = node.get_references();
            refs = refs.saturating_sub(1);

            let mut new_node;
            match node.get_variant() {
                NodeVariant::Branch(b) => {
                    if refs == 0 {
                        let zero = *b.get_zero();
                        let one = *b.get_one();
                        nodes.push_back(zero);
                        nodes.push_back(one);
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = M::Node::new(NodeVariant::Branch(b));
                }
                NodeVariant::Leaf(l) => {
                    if refs == 0 {
                        let data = *l.get_data();
                        nodes.push_back(data);
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = M::Node::new(NodeVariant::Leaf(l));
                }
                NodeVariant::Data(d) => {
                    if refs == 0 {
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = M::Node::new(NodeVariant::Data(d));
                }
            }

            new_node.set_references(refs);
            self.db.insert(node_location, new_node)?;
        }
        self.db.batch_write()?;

        Ok(())
    }

    /// Generates an inclusion proof.  The proof consists of a list of hashes beginning with the key/value
    /// pair and traveling up the tree until the level below the root is reached.
    /// # Errors
    /// `Exception` generated when an invalid state is encountered during tree traversal.
    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &Array<N>,
        key: Array<N>,
    ) -> BinaryMerkleTreeResult<Vec<(Array<N>, bool)>> {
        let mut nodes = VecDeque::with_capacity(self.depth);
        nodes.push_front(*root);

        let mut proof = Vec::with_capacity(self.depth);

        let mut found_leaf = false;
        let mut depth = 0;
        while let Some(location) = nodes.pop_front() {
            if depth > self.depth {
                return Err(MerkleBitError::DepthExceeded(depth));
            }
            depth = depth.saturating_add(1);

            let node = self
                .db
                .get_node(location)?
                .ok_or(CorruptTreeError::NoNodeFromDB)?;
            match node.get_variant() {
                NodeVariant::Branch(b) => {
                    if found_leaf {
                        return Err(CorruptTreeError::MisplacedLeaf.into());
                    }
                    let index = b.get_split_index();
                    let b_key = b.get_key();
                    let min_split_index = calc_min_split_index(&[key], b_key)?;
                    let keys = &[key];
                    let descendants = check_descendants(keys, index, b_key, min_split_index)?;
                    if descendants.is_empty() {
                        return Err(MerkleBitError::KeyNotPresent);
                    }

                    if choose_zero(key, index)? {
                        proof.push((*b.get_one(), true));
                        nodes.push_back(*b.get_zero());
                    } else {
                        proof.push((*b.get_zero(), false));
                        nodes.push_back(*b.get_one());
                    }
                }
                NodeVariant::Leaf(l) => {
                    if found_leaf {
                        return Err(CorruptTreeError::MisplacedLeaf.into());
                    }
                    if *l.get_key() != key {
                        return Err(MerkleBitError::KeyNotPresent);
                    }

                    let mut leaf_hasher = M::Hasher::new(location.len());
                    leaf_hasher.update(b"l");
                    leaf_hasher.update(&l.get_key()[..]);
                    leaf_hasher.update(&l.get_data()[..]);
                    let leaf_node_location = leaf_hasher.finalize();

                    proof.push((leaf_node_location, false));
                    nodes.push_back(*l.get_data());
                    found_leaf = true;
                }
                NodeVariant::Data(d) => {
                    if !found_leaf {
                        return Err(CorruptTreeError::MisplacedLeaf.into());
                    }

                    let mut data_hasher = M::Hasher::new(location.len());
                    data_hasher.update(b"d");
                    data_hasher.update(&key[..]);
                    data_hasher.update(d.get_value());
                    let data_node_location = data_hasher.finalize();

                    proof.push((data_node_location, false));
                }
            }
        }

        proof.reverse();

        Ok(proof)
    }

    /// Verifies an inclusion proof.
    /// # Errors
    /// `Exception` generated when the given proof is invalid.
    #[inline]
    pub fn verify_inclusion_proof(
        root: &Array<N>,
        key: Array<N>,
        value: &M::Value,
        proof: &[(Array<N>, bool)],
    ) -> BinaryMerkleTreeResult<()> {
        if proof.len() < 2 {
            return Err(MerkleBitError::ProofTooShort);
        }

        let key_len = root.len();

        let mut data_hasher = M::Hasher::new(key_len);
        data_hasher.update(b"d");
        data_hasher.update(&key[..]);
        data_hasher.update(&value.encode()?);
        let data_hash = data_hasher.finalize();

        if data_hash != proof[0].0 {
            return Err(MerkleBitError::InvalidProof);
        }

        let mut leaf_hasher = M::Hasher::new(key_len);
        leaf_hasher.update(b"l");
        leaf_hasher.update(&key[..]);
        leaf_hasher.update(&data_hash[..]);
        let leaf_hash = leaf_hasher.finalize();

        if leaf_hash != proof[1].0 {
            return Err(MerkleBitError::InvalidProof);
        }

        let mut current_hash = leaf_hash;

        for item in proof.iter().skip(2) {
            let mut branch_hasher = M::Hasher::new(key_len);
            branch_hasher.update(b"b");
            if item.1 {
                branch_hasher.update(&current_hash[..]);
                branch_hasher.update(&item.0[..]);
            } else {
                branch_hasher.update(&item.0[..]);
                branch_hasher.update(&current_hash[..]);
            }
            let branch_hash = branch_hasher.finalize();
            current_hash = branch_hash;
        }

        if *root != current_hash {
            return Err(MerkleBitError::InvalidProof);
        }

        Ok(())
    }

    /// Gets a single key from the tree.
    /// # Errors
    /// `Exception` generated from encountering an invalid state during tree traversal.
    #[inline]
    pub fn get_one(
        &self,
        root: &Array<N>,
        key: &Array<N>,
    ) -> BinaryMerkleTreeResult<Option<M::Value>> {
        let mut nodes = VecDeque::with_capacity(3);
        nodes.push_front(*root);

        let mut found_leaf = false;
        let mut depth = 0;

        while let Some(location) = nodes.pop_front() {
            if depth > self.depth {
                return Err(MerkleBitError::DepthExceeded(depth));
            }
            depth = depth.saturating_add(1);

            if let Some(node) = self.db.get_node(location)? {
                match node.get_variant() {
                    NodeVariant::Branch(b) => {
                        if found_leaf {
                            return Err(CorruptTreeError::MisplacedLeaf.into());
                        }

                        let index = b.get_split_index();
                        let b_key = b.get_key();
                        let min_split_index = calc_min_split_index(&[*key], b_key)?;
                        let keys = &[*key];
                        let descendants = check_descendants(keys, index, b_key, min_split_index)?;
                        if descendants.is_empty() {
                            return Ok(None);
                        }

                        if choose_zero(*key, index)? {
                            nodes.push_back(*b.get_zero());
                        } else {
                            nodes.push_back(*b.get_one());
                        }
                    }
                    NodeVariant::Leaf(l) => {
                        if found_leaf {
                            return Err(CorruptTreeError::MisplacedLeaf.into());
                        }

                        if l.get_key() != key {
                            return Ok(None);
                        }

                        found_leaf = true;
                        nodes.push_back(*l.get_data());
                    }
                    NodeVariant::Data(d) => {
                        if !found_leaf {
                            return Err(CorruptTreeError::MisplacedLeaf.into());
                        }

                        let buffer = d.get_value();
                        let value = M::Value::decode(buffer)?;
                        return Ok(Some(value));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Inserts a single value into a tree.
    /// # Errors
    /// `Exception` generated if an invalid state is encountered during tree traversal.
    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&Array<N>>,
        key: &Array<N>,
        value: &M::Value,
    ) -> BinaryMerkleTreeResult<Array<N>> {
        let mut value_map = HashMap::new();
        value_map.insert(*key, value);

        let leaf_location = self.insert_leaves(&[*key], &value_map)?[0];

        let mut tree_refs = Vec::with_capacity(1);
        let mut key_map = HashMap::new();
        key_map.insert(*key, leaf_location);

        let tree_ref = TreeRef::new(*key, leaf_location, 1, 1);
        tree_refs.push(tree_ref);

        if let Some(root) = previous_root {
            let mut proof_nodes = self.generate_treerefs(root, &mut [*key], &key_map)?;
            tree_refs.append(&mut proof_nodes);
        }

        let new_root = self.create_tree(tree_refs)?;
        Ok(new_root)
    }

    /// Decomposes the tree into its underlying data structures
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    pub fn decompose(self) -> (M::Database, usize) {
        (self.db, self.depth)
    }
}

/// Enum used for splitting nodes into either the left or right path during tree traversal
enum SplitNodeType<'keys, NodeType: Node<N>, const N: usize> {
    /// Used for building the `proof_nodes` variable during tree traversal
    Ref(TreeRef<N>),
    /// Used for appending to the `cell_queue` during tree traversal.
    Cell(TreeCell<'keys, NodeType, N>),
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
pub mod tests {
    use crate::utils::tree_utils::choose_zero;

    use super::*;

    const KEY_LEN: usize = 32;

    #[test]
    fn it_chooses_the_right_branch_easy() -> Result<(), MerkleBitError> {
        let key = [0x0F_u8; KEY_LEN];
        for i in 0..8 {
            let expected_branch = i < 4;
            let branch = choose_zero(key.into(), i)?;
            assert_eq!(branch, expected_branch);
        }
        Ok(())
    }

    #[test]
    fn it_chooses_the_right_branch_medium() -> Result<(), MerkleBitError> {
        {
            let key = [0x55; KEY_LEN];
            for i in 0..8 {
                let expected_branch = i % 2 == 0;
                let branch = choose_zero(key.into(), i)?;
                assert_eq!(branch, expected_branch);
            }
        }

        let key = [0xAA; KEY_LEN];
        for i in 0..8 {
            let expected_branch = i % 2 != 0;
            let branch = choose_zero(key.into(), i)?;
            assert_eq!(branch, expected_branch);
        }

        Ok(())
    }

    #[test]
    fn it_chooses_the_right_branch_hard() -> Result<(), MerkleBitError> {
        {
            let key = [0x68; KEY_LEN];
            for i in 0..8 {
                let expected_branch = !(i == 1 || i == 2 || i == 4);
                let branch = choose_zero(key.into(), i)?;
                assert_eq!(branch, expected_branch);
            }
        }

        let key = [0xAB; KEY_LEN];
        for i in 0..8 {
            let expected_branch = !(i == 0 || i == 2 || i == 4 || i == 6 || i == 7);
            let branch = choose_zero(key.into(), i)?;
            assert_eq!(branch, expected_branch);
        }

        Ok(())
    }

    #[test]
    fn it_splits_an_all_zeros_sorted_list_of_pairs() -> Result<(), MerkleBitError> {
        // The complexity of these tests result from the fact that getting a key and splitting the
        // tree should not require any copying or moving of memory.
        let zero_key = Array::from([0x00_u8; KEY_LEN]);
        let key_vec = vec![
            zero_key, zero_key, zero_key, zero_key, zero_key, zero_key, zero_key, zero_key,
            zero_key, zero_key,
        ];
        let keys = key_vec;

        let result = split_pairs(&keys, 0)?;
        assert_eq!(result.0.len(), 10);
        assert_eq!(result.1.len(), 0);
        for &res in result.0 {
            assert_eq!(res, Array::from([0x00_u8; KEY_LEN]));
        }

        Ok(())
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() -> Result<(), MerkleBitError> {
        let one_key = Array::from([0xFF_u8; KEY_LEN]);
        let keys = vec![
            one_key, one_key, one_key, one_key, one_key, one_key, one_key, one_key, one_key,
            one_key,
        ];
        let result = split_pairs(&keys, 0)?;
        assert_eq!(result.0.len(), 0);
        assert_eq!(result.1.len(), 10);
        for &res in result.1 {
            assert_eq!(res, Array::from([0xFF_u8; KEY_LEN]));
        }
        Ok(())
    }

    #[test]
    fn it_splits_an_even_length_sorted_list_of_pairs() -> Result<(), MerkleBitError> {
        let zero_key = Array::from([0x00_u8; KEY_LEN]);
        let one_key = Array::from([0xFF_u8; KEY_LEN]);
        let keys = vec![
            zero_key, zero_key, zero_key, zero_key, zero_key, one_key, one_key, one_key, one_key,
            one_key,
        ];
        let result = split_pairs(&keys, 0)?;
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 5);
        for &res in result.0 {
            assert_eq!(res, Array::from([0x00_u8; KEY_LEN]));
        }
        for &res in result.1 {
            assert_eq!(res, Array::from([0xFF_u8; KEY_LEN]));
        }
        Ok(())
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_zeros() -> Result<(), MerkleBitError>
    {
        let zero_key = Array::from([0x00_u8; KEY_LEN]);
        let one_key = Array::from([0xFF_u8; KEY_LEN]);
        let keys = vec![
            zero_key, zero_key, zero_key, zero_key, zero_key, zero_key, one_key, one_key, one_key,
            one_key, one_key,
        ];
        let result = split_pairs(&keys, 0)?;
        assert_eq!(result.0.len(), 6);
        assert_eq!(result.1.len(), 5);
        for &res in result.0 {
            assert_eq!(res, Array::from([0x00_u8; KEY_LEN]));
        }
        for &res in result.1 {
            assert_eq!(res, Array::from([0xFF_u8; KEY_LEN]));
        }

        Ok(())
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_ones() -> Result<(), MerkleBitError> {
        let zero_key = Array::from([0x00_u8; KEY_LEN]);
        let one_key = Array::from([0xFF_u8; KEY_LEN]);
        let keys = vec![
            zero_key, zero_key, zero_key, zero_key, zero_key, one_key, one_key, one_key, one_key,
            one_key, one_key,
        ];

        let result = split_pairs(&keys, 0)?;
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 6);
        for &res in result.0 {
            assert_eq!(res, Array::from([0x00_u8; KEY_LEN]));
        }
        for &res in result.1 {
            assert_eq!(res, Array::from([0xFF_u8; KEY_LEN]));
        }

        Ok(())
    }
}
