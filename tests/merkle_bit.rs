#[cfg(test)]
#[cfg(feature = "default_tree")]
pub mod integration_tests {
    extern crate rocksdb;

    use std::path::PathBuf;
    use std::error::Error;
    use std::fs::remove_dir_all;

    use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
    use rocksdb::{DB, WriteBatch};
    use starling::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
    use starling::tree::{TreeBranch, TreeData, TreeLeaf, TreeNode};
    use starling::traits::{Database, Decode, Encode, Hasher};
    use std::cmp::Ordering;

    #[test]
    fn it_works_with_a_real_database() {
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        let path = PathBuf::from("db");
        {
            let key = vec![0xAAu8];
            let values = vec![data.as_ref()];
            let mut tree = RocksTree::open(&path);
            let root = tree.insert(None, vec![key.as_ref()], &values).unwrap();
            retrieved_value = tree.get(&root, vec![&key[..]]).unwrap();
            tree.remove(&root).unwrap();
            removed_retrieved_value = tree.get(&root, vec![&key[..]]).unwrap();
        }
        remove_dir_all(&path).unwrap();
        assert_eq!(retrieved_value, vec![Some(data)]);
        assert_eq!(removed_retrieved_value, vec![None]);
    }

    struct RocksDB {
        db: DB,
        pending_inserts: Vec<(Vec<u8>, Vec<u8>)>
    }

    impl RocksDB {
        pub fn new(db: DB) -> Self {
            Self {
                db,
                pending_inserts: Vec::with_capacity(64)
            }
        }
    }

    impl Database for RocksDB {
        type NodeType = TreeNode;
        type EntryType = (Vec<u8>, Vec<u8>);

        fn open(path: &PathBuf) -> Result<Self, Box<Error>> {
            Ok(RocksDB::new(DB::open_default(path)?))
        }

        fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
            if let Some(buffer) = self.db.get(key)? {
                Ok(Some(Self::NodeType::decode(buffer.as_ref())?))
            } else {
                Ok(None)
            }
        }

        fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
            let serialized = value.encode()?;
            self.pending_inserts.push((key.to_vec(), serialized));
            Ok(())
        }

        fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
            Ok(self.db.delete(key)?)
        }

        fn batch_write(&mut self) -> Result<(), Box<Error>> {
            let mut batch = WriteBatch::default();
            for pair in &self.pending_inserts {
                batch.put(&pair.0, &pair.1)?;
            }
            self.db.write(batch)?;
            self.pending_inserts.clear();
            Ok(())
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
}
