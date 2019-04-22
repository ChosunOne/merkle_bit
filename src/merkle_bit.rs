use std::cell::RefCell;
use std::collections::{BinaryHeap, VecDeque};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;

use crate::traits::{Branch, Data, Database, Decode, Encode, Exception, Hasher, Leaf, Node, NodeVariant};
use crate::constants::KEY_LEN;
use crate::utils::tree_ref_wrapper::TreeRefWrapper;
use crate::utils::tree_ref::TreeRef;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "use_hashbrown"))]
use std::collections::HashMap;
use crate::utils::tree_cell::TreeCell;
use crate::utils::tree_utils::{split_pairs, check_descendants, calc_min_split_index, fast_log_2};

/// A generic Result from an operation involving a MerkleBIT
pub type BinaryMerkleTreeResult<T> = Result<T, Exception>;

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
    LeafType: Leaf,
    DataType: Data,
    NodeType: Node<BranchType, LeafType, DataType>,
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
    LeafType: Leaf,
    DataType: Data,
    NodeType: Node<BranchType, LeafType, DataType>,
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
        root_hash: &[u8; KEY_LEN],
        keys: &mut [&'a [u8; KEY_LEN]],
    ) -> BinaryMerkleTreeResult<HashMap<&'a [u8; KEY_LEN], Option<ValueType>>> {
        let mut leaf_map = HashMap::new();
        for key in keys.iter() {
            leaf_map.insert(*key, None);
        }

        if keys.is_empty() {
            return Ok(leaf_map);
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
            let tree_cell = cell_queue
                .pop_front()
                .expect("Cell queue should not be empty.");

            if tree_cell.depth > self.depth {
                return Err(Exception::new("Depth of merkle tree exceeded"));
            }

            let node = tree_cell.node;

            match node.get_variant() {
                NodeVariant::Branch(branch) => {
                    let (_, zero, one, branch_split_index, branch_key) = branch.deconstruct();
                    let min_split_index =
                        calc_min_split_index(&tree_cell.keys, &branch_key)?;
                    let descendants = check_descendants(
                        tree_cell.keys,
                        branch_split_index,
                        &branch_key,
                        min_split_index,
                    );
                    if descendants.is_empty() {
                        continue;
                    }

                    let (zeros, ones) = split_pairs(&descendants, branch_split_index);

                    if let Some(o) = self.db.get_node(&one)? {
                        let one_node = o;
                        if !ones.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                                ones,
                                one_node,
                                tree_cell.depth + 1,
                            );
                            cell_queue.push_front(new_cell);
                        }
                    }

                    if let Some(z) = self.db.get_node(&zero)? {
                        let zero_node = z;
                        if !zeros.is_empty() {
                            let new_cell = TreeCell::new::<BranchType, LeafType, DataType>(
                                zeros,
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
                            if let Ok(index) = keys.binary_search(&n.get_key()) {
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
        previous_root: Option<&[u8; KEY_LEN]>,
        keys: &mut [&[u8; KEY_LEN]],
        values: &mut [&ValueType],
    ) -> BinaryMerkleTreeResult<[u8; KEY_LEN]> {
        if keys.len() != values.len() {
            return Err(Exception::new("Keys and values have different lengths"));
        }

        if keys.is_empty() || values.is_empty() {
            return Err(Exception::new("Keys or values are empty"));
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
            let mut tree_ref_key = [0; KEY_LEN];
            tree_ref_key.copy_from_slice(&key[..]);
            let tree_ref = TreeRef::new(tree_ref_key, loc, 1);
            tree_refs.push(tree_ref);
        }

        if let Some(root) = previous_root {
            self.generate_treerefs(root, keys, &mut tree_refs)?;
        }

        let new_root = self.create_tree(tree_refs)?;
        Ok(new_root)
    }

    fn generate_treerefs(
        &mut self,
        root: &[u8; KEY_LEN],
        keys: &mut [&[u8; KEY_LEN]],
        tree_refs: &mut Vec<TreeRef>,
    ) -> BinaryMerkleTreeResult<()> {
        // Nodes that form the merkle proof for the new tree
        let mut proof_nodes = Vec::with_capacity(keys.len());

        let root_node = if let Some(m) = self.db.get_node(root)? {
            m
        } else {
            return Err(Exception::new("Could not find root"));
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

                    let mut leaf_hasher = HasherType::new(KEY_LEN);
                    leaf_hasher.update(b"l");
                    leaf_hasher.update(&key);
                    leaf_hasher.update(&data);
                    let mut location = [0; KEY_LEN];
                    location.copy_from_slice(&leaf_hasher.finalize());

                    let mut update = false;

                    // Check if we are updating an existing value
                    if let Ok(index) = tree_refs.binary_search_by(|x| x.key[..].cmp(&key)) {
                        if tree_refs[index].location[..] == location {
                            update = true;
                        } else {
                            continue;
                        }
                    }

                    if let Some(mut l) = self.db.get_node(&location)? {
                        let refs = l.get_references() + 1;
                        l.set_references(refs);
                        self.db.insert(location, l)?;
                    } else {
                        return Err(Exception::new("Corrupt merkle tree"));
                    }

                    if update {
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

            let mut branch_hasher = HasherType::new(KEY_LEN);
            branch_hasher.update(b"b");
            branch_hasher.update(&branch_zero);
            branch_hasher.update(&branch_one);
            let location = branch_hasher.finalize();

            let min_split_index = calc_min_split_index(&tree_cell.keys, &branch_key)?;

            let split;
            let mut descendants = &tree_cell.keys[..];

            if min_split_index < branch_split_index {
                descendants = check_descendants(
                    &tree_cell.keys,
                    branch_split_index,
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
                    self.db.insert(*tree_ref.location, new_node)?;
                    proof_nodes.push(tree_ref);
                    continue;
                }
            }

            split = split_pairs(descendants, branch_split_index);
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
                    let mut other_key = [0; KEY_LEN];
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
                        _ => {
                            return Err(Exception::new("Corrupt merkle tree"));
                        }
                    }
                    new_one_node.set_references(refs);
                    self.db.insert(branch_one, new_one_node)?;
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
                    let mut other_key = [0; KEY_LEN];
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
                        _ => {
                            return Err(Exception::new("Corrupt merkle tree"));
                        }
                    }
                    new_zero_node.set_references(refs);
                    self.db.insert(branch_zero, new_zero_node)?;
                    let tree_ref = TreeRef::new(other_key, branch_zero, count);
                    proof_nodes.push(tree_ref);
                }
            }
        }

        tree_refs.append(&mut proof_nodes);
        Ok(())
    }

    fn insert_leaves(
        &mut self,
        keys: &[&[u8; KEY_LEN]],
        values: &[&ValueType],
    ) -> BinaryMerkleTreeResult<Vec<[u8; KEY_LEN]>> {
        let mut nodes = Vec::with_capacity(keys.len());
        for i in 0..keys.len() {
            // Create data node
            let mut data = DataType::new();
            data.set_value(&values[i].encode()?);

            let mut data_hasher = HasherType::new(KEY_LEN);
            data_hasher.update(b"d");
            data_hasher.update(keys[i]);
            data_hasher.update(data.get_value());
            let data_node_location = data_hasher.finalize();

            let mut data_node = NodeType::new(NodeVariant::Data(data));
            data_node.set_references(1);

            // Create leaf node
            let mut leaf = LeafType::new();
            leaf.set_data(data_node_location);
            let mut leaf_key = [0; KEY_LEN];
            leaf_key.copy_from_slice(keys[i]);
            leaf.set_key(leaf_key);

            let mut leaf_hasher = HasherType::new(KEY_LEN);
            leaf_hasher.update(b"l");
            leaf_hasher.update(keys[i]);
            leaf_hasher.update(&leaf.get_data()[..]);
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

            self.db.insert(data_node_location, data_node)?;
            self.db.insert(leaf_node_location, leaf_node)?;

            let mut location = [0; KEY_LEN];
            location.copy_from_slice(&leaf_node_location);
            nodes.push(location);
        }
        Ok(nodes)
    }

    fn create_tree(&mut self, mut tree_refs: Vec<TreeRef>) -> BinaryMerkleTreeResult<[u8; KEY_LEN]> {
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
            .map(|x| Rc::new(RefCell::new(TreeRefWrapper::Raw(Rc::new(RefCell::new(x))))))
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
                    (j * 8) as u8 + (7 - fast_log_2(xor_key) as u8);

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
                let mut branch_hasher = HasherType::new(KEY_LEN);
                branch_hasher.update(b"b");
                branch_hasher.update(&tree_ref_location[..]);
                branch_hasher.update(&next_tree_ref_location[..]);
                let branch_node_location_vec = branch_hasher.finalize();
                let mut branch_node_location_arr = [0u8; KEY_LEN];
                branch_node_location_arr.copy_from_slice(&branch_node_location_vec);
                branch_node_location = Rc::new(branch_node_location_arr);

                count = tree_ref_count + next_tree_ref_count;
                let mut branch_zero = [0; KEY_LEN];
                branch_zero.copy_from_slice(&tree_ref_location[..]);

                let mut branch_one = [0; KEY_LEN];
                branch_one.copy_from_slice(&next_tree_ref_location[..]);

                let mut branch_key = [0; KEY_LEN];
                branch_key.copy_from_slice(&tree_ref_key[..]);

                branch.set_zero(branch_zero);
                branch.set_one(branch_one);
                branch.set_count(count);
                branch.set_split_index(split_index);
                branch.set_key(branch_key);
            }

            let mut branch_node = NodeType::new(NodeVariant::Branch(branch));
            branch_node.set_references(1);

            self.db.insert(*branch_node_location, branch_node)?;

            next_tree_ref_wrapper
                .borrow_mut()
                .set_tree_ref_key(Rc::clone(&tree_ref_key));
            next_tree_ref_wrapper
                .borrow_mut()
                .set_tree_ref_location(Rc::clone(&branch_node_location));
            next_tree_ref_wrapper.borrow_mut().set_tree_ref_count(count);

            *tree_ref_wrapper.borrow_mut() = TreeRefWrapper::Ref(next_tree_ref_wrapper);

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
    pub fn remove(&mut self, root_hash: &[u8; KEY_LEN]) -> BinaryMerkleTreeResult<()> {
        let mut nodes = VecDeque::with_capacity(128);
        nodes.push_front(*root_hash);

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
                        let zero = *b.get_zero();
                        let one = *b.get_one();
                        nodes.push_back(zero);
                        nodes.push_back(one);
                        self.db.remove(&node_location)?;
                        continue;
                    }
                    new_node = NodeType::new(NodeVariant::Branch(b))
                }
                NodeVariant::Leaf(l) => {
                    if refs == 0 {
                        let data = *l.get_data();
                        nodes.push_back(data);
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
            self.db.insert(node_location, new_node)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::tree_utils::choose_zero;

    #[test]
    fn it_chooses_the_right_branch_easy() {
        let key = [0x0F; KEY_LEN];
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
        let key = [0x55; KEY_LEN];
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
        let key = [0xAA; KEY_LEN];
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
        let key = [0x68; KEY_LEN];
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

        let key = [0xAB; KEY_LEN];
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
        let zero_key = [0x00u8; KEY_LEN];
        let key_vec = vec![
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
        ];
        let keys = key_vec;

        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 10);
        assert_eq!(result.1.len(), 0);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8; KEY_LEN]);
        }
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() {
        let one_key = [0xFFu8; KEY_LEN];
        let keys = vec![
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
        ];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 0);
        assert_eq!(result.1.len(), 10);
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8; KEY_LEN]);
        }
    }

    #[test]
    fn it_splits_an_even_length_sorted_list_of_pairs() {
        let zero_key = [0x00u8; KEY_LEN];
        let one_key = [0xFFu8; KEY_LEN];
        let keys = vec![
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
        ];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 5);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8; KEY_LEN]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8; KEY_LEN]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_zeros() {
        let zero_key = [0x00u8; KEY_LEN];
        let one_key = [0xFFu8; KEY_LEN];
        let keys = vec![
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
        ];
        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 6);
        assert_eq!(result.1.len(), 5);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8; KEY_LEN]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8; KEY_LEN]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_ones() {
        let zero_key = [0x00u8; KEY_LEN];
        let one_key = [0xFFu8; KEY_LEN];
        let keys = vec![
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &zero_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
            &one_key,
        ];

        let result = split_pairs(&keys, 0);
        assert_eq!(result.0.len(), 5);
        assert_eq!(result.1.len(), 6);
        for i in 0..result.0.len() {
            assert_eq!(*result.0[i], [0x00u8; KEY_LEN]);
        }
        for i in 0..result.1.len() {
            assert_eq!(*result.1[i], [0xFFu8; KEY_LEN]);
        }
    }
}
