#[macro_use]
extern crate criterion;

use criterion::Criterion;
use starling::tree::HashTree;
use starling::merkle_bit::{MerkleBIT,NodeVariant,BinaryMerkleTreeResult};
use starling::traits::{Database,Branch,Leaf,Data,Node,Exception,Encode,Decode,Hasher};
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::path::PathBuf;
use std::error::Error;

fn empty_tree_insert_benchmark(c: &mut Criterion){
    c.bench_function_over_inputs("Hash Tree Empty Insert", move |b,index| {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(100, &mut rng);
        let key_values = prepare.0;
        let mut keys = vec![];    
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let db = MockDB::new(HashMap::new());
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, HasherContainer, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        b.iter(|| bmt.insert(None, keys[0..*index].to_vec(), &data[0..*index]))
    },vec![1,10,100]);
}
fn existing_tree_insert_benchmark(c: &mut Criterion) {
        c.bench_function_over_inputs("Hash Tree Non Empty Insert", move |b,index| {
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
        let mut bmt: MerkleBIT<MockDB, ProtoBranch, ProtoLeaf, ProtoData, ProtoMerkleNode, HasherContainer, Vec<u8>, Vec<u8>> = MerkleBIT::from_db(db, 16).unwrap();
        let root_hash = bmt.insert(None, keys.clone(), &data).unwrap();
             let second = prepare_inserts(100, &mut rng);
             let mut second_data = vec![];
             let mut second_keys =vec![];
             let keys_2 = second.0;
             let data_2 = second.1;
            for i in 0..data_2.len() {
            second_data.push(data_2[i].as_ref());
            second_keys.push(keys_2[i].as_ref());
        }
            b.iter(|| bmt.insert(Some(&root_hash),second_keys[0..*index].to_vec(),&second_data[0..*index]))
        },vec![1,10,100]);
}
// fn get_from_tree_benchmark(c: &mut Criterion) {}
// fn get_from_tree_worst_case_benchmark(c: &mut Criterion) {}

criterion_group!(benches, empty_tree_insert_benchmark,existing_tree_insert_benchmark);
criterion_main!(benches);

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
//helper functions from tests for easier test setup
    struct MockDB {
        map: HashMap<Vec<u8>, ProtoMerkleNode>,
        pending_inserts: Vec<(Vec<u8>, ProtoMerkleNode)>,
    }

    impl MockDB {
        pub fn new(map: HashMap<Vec<u8>, ProtoMerkleNode>) -> MockDB {
            MockDB {
                map,
                pending_inserts: Vec::with_capacity(64),
            }
        }
    }

    impl Database for MockDB {
        type NodeType = ProtoMerkleNode;
        type EntryType = (Vec<u8>, Self::NodeType);

        fn open(_path: &PathBuf) -> Result<MockDB, Box<Error>> {
            Ok(MockDB::new(HashMap::new()))
        }

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
        split_index: u32,
    }

    impl ProtoBranch {
        fn new() -> ProtoBranch {
            ProtoBranch {
                count: 0,
                zero: vec![],
                one: vec![],
                split_index: 0,
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
        data: Vec<u8>,
    }

    impl ProtoLeaf {
        fn new() -> ProtoLeaf {
            ProtoLeaf {
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
        node: Option<NodeVariant<ProtoBranch, ProtoLeaf, ProtoData>>,
    }

    impl ProtoMerkleNode {
        fn new() -> ProtoMerkleNode {
            ProtoMerkleNode {
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
        fn set_key(&mut self, _key: &[u8]) {}
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
                }
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
    pub struct HasherContainer{
        inner: Vec<u8>
    }
    impl Hasher for HasherContainer {
        type HashType = HasherContainer;
        type HashResultType = Vec<u8>;
        fn new(size: usize) -> Self::HashType {
            Self{inner:Vec::with_capacity(size)}
        }
        fn update(&mut self, data: &[u8]) {
            for i in 0..data.len() {
                self.inner.push(data[i]);
            }
        }
        fn finalize(self) -> Self::HashResultType {
            self.inner
        }
    }
