use std::collections::VecDeque;
use std::path::PathBuf;
use std::error::Error;

use common::{Encode, Exception, Decode};
use common::traits::{Branch, Data, Hasher, IDB, IdentifyNode, Leaf};

pub type BinaryMerkleTreeResult<T> = Result<T, Box<Error>>;

pub enum NodeVariant<BranchType, LeafType, DataType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data {
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType)
}

enum TreeBranch {
    Zero,
    One
}

struct SplitPairs<'a> {
    zeros: &'a [&'a [u8]],
    ones: &'a [&'a [u8]]
}

struct Foo<'a, NodeType> {
    keys: &'a [&'a [u8]],
    node: NodeType,
    depth: usize
}

impl<'a> SplitPairs<'a> {
    pub fn new(zeros: &'a [&'a [u8]], ones: &'a [&'a [u8]]) -> SplitPairs<'a> {
        SplitPairs {
            zeros: &zeros,
            ones: &ones
        }
    }
}

impl<'a, 'b, NodeType> Foo<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(keys: &'a [&'a [u8]], node: NodeType, depth: usize) -> Foo<'a, NodeType>
        where BranchType: Branch,
              LeafType: Leaf,
              DataType: Data {
        Foo {
            keys,
            node,
            depth
        }
    }
}

fn choose_branch(key: &[u8], bit: usize) -> TreeBranch {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    if extracted_bit == 0 {
        return TreeBranch::Zero
    } else {
        return TreeBranch::One
    }
}

fn split_pairs<'a>(sorted_pairs: &'a [&'a [u8]], bit: usize) ->  SplitPairs<'a> {

    if let TreeBranch::Zero = choose_branch(sorted_pairs[sorted_pairs.len() - 1], bit) {
        return SplitPairs::new(&sorted_pairs[0..sorted_pairs.len()], &sorted_pairs[0..0])
    }

    if let TreeBranch::One = choose_branch(sorted_pairs[0], bit) {
        return SplitPairs::new(&sorted_pairs[0..0], &sorted_pairs[0..sorted_pairs.len()])
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();
    let mut iterations = 0;
    while max - min > 1 {
        let bisect = (max - min) / 2 + min;
        match choose_branch(sorted_pairs[bisect], bit) {
            TreeBranch::Zero => min = bisect,
            TreeBranch::One =>  max = bisect
        }
    }

    SplitPairs::new(&sorted_pairs[0..max], &sorted_pairs[max..sorted_pairs.len()])
}

pub struct BinaryMerkleTree<DatabaseType, BranchType, LeafType, DataType, NodeType, ValueType>
    where DatabaseType: IDB<NodeType = NodeType, ValueType = ValueType>,
          BranchType: Branch,
          LeafType: Leaf,
          DataType: Data,
          NodeType: IdentifyNode<BranchType, LeafType, DataType> + Encode + Decode {
    db: DatabaseType,
    depth: usize,
    branch: Option<BranchType>,
    leaf: Option<LeafType>,
    data: Option<DataType>,
    node: Option<NodeType>
}

impl<DatabaseType, BranchType, LeafType, DataType, NodeType, ReturnType> BinaryMerkleTree<DatabaseType, BranchType, LeafType, DataType, NodeType, ReturnType>
    where DatabaseType: IDB<NodeType = NodeType, ValueType = ReturnType>,
          BranchType: Branch,
          LeafType: Leaf,
          DataType: Data,
          NodeType: IdentifyNode<BranchType, LeafType, DataType> + Encode + Decode,
          ReturnType: Decode {
    pub fn new(path: PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = DatabaseType::open(path)?;
        Ok(Self {
            db,
            depth,
            branch: None,
            leaf: None,
            data: None,
            node: None
        })
    }

    pub fn from_db(db: DatabaseType, depth: usize) -> BinaryMerkleTreeResult<Self> {
        Ok(Self {
            db,
            depth,
            branch: None,
            leaf: None,
            data: None,
            node: None
        })
    }

    pub fn get<HashResultType>(&self, root_hash: &HashResultType, keys: &[&[u8]]) -> BinaryMerkleTreeResult<Vec<ReturnType>>
        where HashResultType: AsRef<[u8]> {

        let root_node;
        if let Some(n) = self.db.get_node(root_hash.as_ref())? {
            root_node = n;
        } else {
            return Err(Box::new(Exception::new("Failed to retrieve data")))
        }

        let mut data_nodes: Vec<DataType> = Vec::with_capacity(keys.len());

        let mut foo_queue: VecDeque<Foo<NodeType>> = VecDeque::with_capacity(2.0f64.powf(self.depth as f64) as usize);
        let root_foo: Foo<NodeType> = Foo::new::<BranchType, LeafType, DataType>(keys, root_node, 0);

        foo_queue.push_back(root_foo);

        while foo_queue.len() > 0 {
            let foo;
            match foo_queue.pop_front() {
                Some(f) => foo = f,
                None => return Err(Box::new(Exception::new("Empty Foo")))
            }



            if foo.depth > self.depth {
                return Err(Box::new(Exception::new("Depth of merkle tree exceeded")))
            }

            match foo.node.get_variant()? {
                NodeVariant::Branch(n) => {
                    let split = split_pairs(foo.keys, foo.depth);
                    if let Some(z) = self.db.get_node(n.get_zero())? {
                        let zero_node = z;
                        let new_foo = Foo::new::<BranchType, LeafType, DataType>(split.zeros, zero_node, foo.depth + 1);
                        foo_queue.push_back(new_foo);
                    } else {
                        return Err(Box::new(Exception::new("Corrupt merkle tree")))
                    }
                    if let Some(o) = self.db.get_node(n.get_one())? {
                        let one_node = o;
                        let new_foo = Foo::new::<BranchType, LeafType, DataType>(split.ones, one_node, foo.depth + 1);
                        foo_queue.push_back(new_foo);
                    } else {
                        return Err(Box::new(Exception::new("Corrupt merkle tree")))
                    }
                },
                NodeVariant::Leaf(n) => {
                    if foo.keys.len() == 0 {
                        return Err(Box::new(Exception::new("No key with which to match the leaf key")))
                    }
                    if n.get_key() != foo.keys[0] {
                        return Err(Box::new(Exception::new("Given key does not match leaf key")))
                    }
                    let data = n.get_data();
                    let new_node;
                    if let Some(e) = self.db.get_node(data)? {
                        new_node = e;
                    } else {
                        return Err(Box::new(Exception::new("Corrupt merkle tree")))
                    }
                    let new_foo = Foo::new::<BranchType, LeafType, DataType>(foo.keys, new_node, foo.depth + 1);
                    foo_queue.push_back(new_foo);
                },
                NodeVariant::Data(n) => {data_nodes.push(n)}
            }
        }

        let mut values = Vec::with_capacity(data_nodes.len());


        for data_node in data_nodes {
            if let Some(v) = self.db.get_value(data_node.get_value())? {
                values.push(v);
            } else {
                return Err(Box::new(Exception::new("Failed to find requested key")))
            }
        }

        Ok(values)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use serialization::state::{MerkleNode as ProtoMerkleNode,
                               MerkleNode_oneof_node::branch as ProtoMerkleNodeBranch,
                               MerkleNode_oneof_node::data as ProtoMerkleNodeData,
                               MerkleNode_oneof_node::leaf as ProtoMerkleNodeLeaf,
                               Branch as ProtoBranch,
                               Leaf as ProtoLeaf,
                               Data as ProtoData};

    use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
    use protobuf::Message as ProtoMessage;
    use std::collections::HashMap;
    use rand::{Rng, StdRng, SeedableRng};
    use util::hash::hash;

    struct MockDB {
        node_map: HashMap<Vec<u8>, ProtoMerkleNode>,
        value_map: HashMap<Vec<u8>, Vec<u8>>
    }

    impl MockDB {
        pub fn new(node_map: HashMap<Vec<u8>, ProtoMerkleNode>, value_map: HashMap<Vec<u8>, Vec<u8>>) -> MockDB {
            MockDB {
                node_map,
                value_map
            }
        }

        pub fn update_node_map(&mut self, map: HashMap<Vec<u8>, ProtoMerkleNode>) {
            self.node_map = map;
        }

        pub fn update_value_map(&mut self, map: HashMap<Vec<u8>, Vec<u8>>) {
            self.value_map = map;
        }
    }


    impl IDB for MockDB {
        type NodeType = ProtoMerkleNode;
        type ValueType = Vec<u8>;

        fn open(path: PathBuf) -> Result<MockDB, Box<Error>> {
            Ok(MockDB::new(HashMap::new(), HashMap::new()))
        }

        fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
            if let Some(m) = self.node_map.get(key) {
                return Ok(Some(m.clone()))
            } else {
                return Ok(None)
            }
        }

        fn insert_node(&mut self, key: Vec<u8>, value: Self::NodeType) {
            self.node_map.insert(key, value);
        }

        fn get_value(&self, key: &[u8]) -> Result<Option<Self::ValueType>, Box<Error>> {
            if let Some(v) = self.value_map.get(key) {
                return Ok(Some(v.clone()))
            } else {
                return Ok(None)
            }
        }

        fn insert_value(&mut self, key: Vec<u8>, value: Self::ValueType) {
            self.value_map.insert(key, value);
        }
    }

    impl Hasher for Blake2b {
        type HashType = Blake2b;
        type HashResultType = Blake2bResult;

        fn new(size: usize) -> Self::HashType {
            Blake2b::new(size)
        }
        fn update(&mut self, data: &[u8]) {
            Blake2b::update(self, data)
        }
        fn finalize(self) -> Self::HashResultType {
            Blake2b::finalize(self)
        }
    }

    impl Branch for ProtoBranch {
        fn get_count(&self) -> u64 {
            ProtoBranch::get_count(self)
        }

        fn get_zero(&self) -> &[u8] {
            ProtoBranch::get_zero(self)
        }

        fn get_one(&self) -> &[u8] {
            ProtoBranch::get_one(self)
        }
    }

    impl Leaf for ProtoLeaf {
        fn get_key(&self) -> &[u8] {
            ProtoLeaf::get_key(self)
        }

        fn get_data(&self) -> &[u8] {
            ProtoLeaf::get_data(self)
        }
    }

    impl Data for ProtoData {
        fn get_value(&self) -> &[u8] {
            ProtoData::get_value(self)
        }
    }

    impl IdentifyNode<ProtoBranch, ProtoLeaf, ProtoData> for ProtoMerkleNode {
        fn get_variant(&self)
                       -> BinaryMerkleTreeResult<NodeVariant<ProtoBranch, ProtoLeaf, ProtoData>>
            where ProtoBranch: Branch,
                  ProtoLeaf: Leaf,
                  ProtoData: Data, {
            match self.node {
                Some(ref node_type) => {
                    match node_type {
                        ProtoMerkleNodeBranch(branch) => return Ok(NodeVariant::Branch(branch.clone())),
                        ProtoMerkleNodeData(data) => return Ok(NodeVariant::Data(data.clone())),
                        ProtoMerkleNodeLeaf(leaf) => return Ok(NodeVariant::Leaf(leaf.clone()))
                    }
                },
                None => return Err(Box::new(Exception::new("Failed to distinguish node type")))
            }
        }
    }

    impl Leaf for [u8; 1] {
        fn get_key(&self) -> &[u8] {
            self
        }

        fn get_data(&self) -> &[u8] {
            self
        }
    }

    impl Encode for ProtoMerkleNode {
        fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
            Ok(vec![])
        }
    }

    impl Decode for ProtoMerkleNode {
        fn decode(buffer: &[u8]) -> Result<ProtoMerkleNode, Box<Error>> {
            let mut proto = ProtoMerkleNode::new();
            proto.merge_from_bytes(buffer)?;
            Ok(proto)
        }
    }

    impl Decode for Vec<u8> {
        fn decode(buffer: &[u8]) -> Result<Vec<u8>, Box<Error>> {
            Ok(buffer.to_vec())
        }
    }

    #[test]
    fn it_recognizes_a_hasher() {
        let mut blake = Blake2b::new(32);
        let data = [0u8; 32];
        blake.update(&data);
        let hash = blake.finalize();
        let expected_hash = [
            137, 235,  13, 106, 138, 105, 29, 174,
             44, 209,  94, 208,  54, 153, 49, 206,
             10, 148, 158, 202, 250,  92, 63, 147,
            248,  18,  24,  51, 100, 110, 21, 195];
        assert_eq!(hash.as_bytes(), expected_hash);
    }

    #[test]
    fn it_splits_an_all_zeros_sorted_list_of_pairs() {
        // The complexity of these tests result from the fact that getting a key and splitting the
        // tree should not require any copying or moving of memory.
        let zero_key = vec![0x00];
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..]
        ];
        let keys = &key_vec[..];

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
        let key_vec = vec![
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let keys = &key_vec[..];
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
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let keys = &key_vec[..];
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
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];
        let keys = &key_vec[..];
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
        let key_vec = vec![
            &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..], &zero_key[..],
            &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..], &one_key[..]];

        let keys = &key_vec[..];

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
        let mut db = MockDB::new(HashMap::new(), HashMap::new());

        let proto_data_node_key = insert_data_node(&mut db, vec![0xFF]);
        let proto_root_node_key = insert_leaf_node(&mut db, None, proto_data_node_key.clone());


        let mut bmt = BinaryMerkleTree::from_db(db, 160).unwrap();
        let result = bmt.get(&proto_root_node_key, &[&proto_root_node_key[..]]).unwrap();
        assert_eq!(result, vec![vec![0xFFu8]]);
    }

    #[test]
    #[should_panic]
    fn it_fails_to_get_from_empty_tree() {
        let mut db = MockDB::new(HashMap::new(), HashMap::new());
        let mut rng: StdRng = SeedableRng::from_seed([0x04; 32]);

        let mut root_key_material = [0; 32];
        let mut key_material = [0; 32];

        rng.fill(&mut root_key_material);
        rng.fill(&mut key_material);
        let root_key = hash(&root_key_material, 32);
        let key = hash(&key_material, 32);

        let mut bmt = BinaryMerkleTree::from_db(db, 160).unwrap();
        bmt.get(&root_key, &[&key[..]]).unwrap();
    }

    #[test]
    #[should_panic]
    fn it_fails_to_get_a_nonexistent_item() {
        let mut db = MockDB::new(HashMap::new(), HashMap::new());
        let mut rng: StdRng = SeedableRng::from_seed([0x05; 32]);
        let mut key_material = [0; 32];

        rng.fill(&mut key_material);

        let key = hash(&key_material, 32);

        let data_node_key = insert_data_node(&mut db, vec![0xFF]);
        let leaf_node_key = insert_leaf_node(&mut db, None, data_node_key.clone());
        let mut bmt = BinaryMerkleTree::from_db(db, 160).unwrap();
        bmt.get(&leaf_node_key, &[&key[..]]).unwrap();
    }

    #[test]
    fn it_gets_an_item_from_a_small_tree() {
        let mut db = MockDB::new(HashMap::new(), HashMap::new());
        panic!();
    }

    fn insert_data_node(db: &mut MockDB, value: Vec<u8>) -> Vec<u8> {
        let mut rng: StdRng = SeedableRng::from_seed([0x01; 32]);
        let mut proto_data_node_key_material = [0; 32];
        let mut data_key_material = [0; 32];

        rng.fill(&mut proto_data_node_key_material);
        rng.fill(&mut data_key_material);

        let proto_data_node_key = hash(&proto_data_node_key_material, 32);
        let data_key = hash(&data_key_material, 32);

        let mut proto_data_node = ProtoData::new();
        proto_data_node.set_value(data_key.clone());
        let mut proto_outer_data_node = ProtoMerkleNode::new();
        proto_outer_data_node.set_references(1);
        proto_outer_data_node.set_data(proto_data_node);
        db.insert_node(proto_data_node_key.clone(), proto_outer_data_node);
        db.insert_value(data_key, value);
        proto_data_node_key.clone()
    }

    fn insert_leaf_node(db: &mut MockDB, leaf_key: Option<Vec<u8>>, data_key: Vec<u8>) -> Vec<u8> {
        let mut rng: StdRng = SeedableRng::from_seed([0x02; 32]);
        let mut proto_leaf_node_key_material = [0; 32];

        rng.fill(&mut proto_leaf_node_key_material);

        let proto_leaf_node_key = hash(&proto_leaf_node_key_material, 32);

        let mut proto_leaf_node = ProtoLeaf::new();
        proto_leaf_node.set_data(data_key.clone());
        let leaf_node_key;
        if let Some(key) = leaf_key {
            leaf_node_key = key;
        } else {
            leaf_node_key = proto_leaf_node_key.clone();
        }
        proto_leaf_node.set_key(leaf_node_key.clone());

        let mut proto_outer_leaf_node = ProtoMerkleNode::new();
        proto_outer_leaf_node.set_references(1);
        proto_outer_leaf_node.set_leaf(proto_leaf_node);
        db.insert_node(leaf_node_key.clone(), proto_outer_leaf_node);
        leaf_node_key.clone()
    }

    fn insert_branch_node(db: &mut MockDB, zero_key: Vec<u8>, one_key: Vec<u8>) -> Vec<u8> {
        let mut rng: StdRng = SeedableRng::from_seed([0x03; 32]);
        let mut proto_branch_node_key_material = [0; 32];

        rng.fill(&mut proto_branch_node_key_material);

        let proto_branch_node_key = hash(&proto_branch_node_key_material, 32);

        let mut proto_branch_node = ProtoBranch::new();
        proto_branch_node.set_count(2);
        proto_branch_node.set_zero(zero_key.clone());
        proto_branch_node.set_one(one_key.clone());
        let mut proto_outer_branch_node = ProtoMerkleNode::new();
        proto_outer_branch_node.set_references(1);
        proto_outer_branch_node.set_branch(proto_branch_node);
        db.insert_node(proto_branch_node_key.clone(), proto_outer_branch_node);
        proto_branch_node_key.clone()
    }

    fn build_tree(db: &mut MockDB, num_data_nodes: usize) -> Vec<u8> {
        if num_data_nodes == 0 {
            return vec![]
        }

        let mut rng: StdRng = SeedableRng::from_seed([0x06; 32]);
        let mut data_node_keys = Vec::with_capacity(num_data_nodes);
        for _ in 0..num_data_nodes {
            let value = rng.gen_range(0x00, 0xFF);
            data_node_keys.push(insert_data_node(db, vec![value]));
        }

        let mut depth = (num_data_nodes as f64).log2() as usize;
        let mut leaf_node_keys = Vec::with_capacity(num_data_nodes);
        for i in 0..data_node_keys.len() {
            let key = data_node_keys[i].clone();
            let mut leaf_key = vec![0xFF; 32];
            let index = depth / 8;
            if i % 2 == 0 {
                leaf_key = vec![0x00, 0xFF, 0x00, ];
            } else {
                leaf_key = vec![]
            }
            leaf_node_keys.push(insert_leaf_node(db, None, key));
        }

        if leaf_node_keys.len() == 1 {
            return leaf_node_keys[0].clone()
        }

//        let mut branch_node_keys = Vec::with_capacity(leaf_node_keys.len() / 2);
//

//        for i in (0..leaf_node_keys.len()).step_by(2) {
//
//        }
//
        vec![]
    }

}