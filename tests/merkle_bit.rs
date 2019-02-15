//#![feature(test)] extern crate test;
extern crate rocksdb;

use std::path::PathBuf;
use std::error::Error;
use std::fs::remove_dir_all;

use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use rocksdb::DB;
use starling::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use starling::tree::{TreeBranch, TreeData, TreeLeaf, TreeNode};
use starling::traits::{Database, Encode, Hasher};
use std::cmp::Ordering;

#[test]
fn it_works_with_a_real_database() {
    let key = vec![0xAAu8];
    let data = vec![0xFFu8];
    let values = vec![data.as_ref()];

    let path = PathBuf::from("db");
    let mut tree = RocksTree::open(&path);
    tree.insert(None, vec![key.as_ref()], &values);
    remove_dir_all(&path).unwrap();
}

struct RocksDB(DB);

impl Database for RocksDB {
    type NodeType = TreeNode;
    type EntryType = (Vec<u8>, Vec<u8>);

    fn open(path: &PathBuf) -> Result<Self, Box<Error>> {
        Ok(RocksDB(DB::open_default(path)?))
    }

    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
        unimplemented!();
    }

    fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
        let serialized = value.encode();
        self.0.put(key, &serialized)?;
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        unimplemented!();
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        unimplemented!();
    }
}

struct Blake2bHasher(Blake2b);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Blake2bHashResult(Blake2bResult);

impl Hasher for Blake2bHasher {
    type HashType = Blake2bHasher;
    type HashResultType = Blake2bHashResult;

    fn new(size: usize) -> Self::HashType {
        Blake2bHasher(Blake2b::new(size))
    }

    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self) -> Self::HashResultType {
        Blake2bHashResult(self.0.finalize())
    }
}

impl PartialOrd for Blake2bHashResult {
    fn partial_cmp(&self, other: &Blake2bHashResult) -> Option<Ordering> {
        Some(self.0.as_bytes().cmp(&other.0.as_bytes()))
    }
}

impl AsRef<[u8]> for Blake2bHashResult {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}


struct RocksTree {
    tree: MerkleBIT<
        RocksDB,
        TreeBranch,
        TreeLeaf,
        TreeData,
        TreeNode,
        Blake2bHasher,
        Blake2bHashResult,
        Vec<u8>>
}

impl RocksTree {
    pub fn open(path: &PathBuf) -> Self {
        let db = RocksDB::open(path).unwrap();
        let tree = MerkleBIT::from_db(db, 160).unwrap();
        RocksTree {
            tree
        }
    }

    pub fn get(&self, root_hash: &[u8], keys: Vec<&[u8]>) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&Blake2bHashResult>, keys: Vec<&[u8]>, values: &[&Vec<u8>]) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}