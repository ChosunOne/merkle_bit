#[cfg(test)]
#[cfg(any(feature = "default_tree"))]
pub mod integration_tests {
    extern crate rocksdb;

    use std::path::PathBuf;
    use std::error::Error;
    use std::fs::remove_dir_all;

    use rocksdb::{DB, WriteBatch};
    use starling::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
    use starling::tree_hasher::{TreeHasher, TreeHashResult};
    use starling::tree::tree_branch::TreeBranch;
    use starling::tree::tree_leaf::TreeLeaf;
    use starling::tree::tree_data::TreeData;
    use starling::tree::tree_node::TreeNode;
    use starling::traits::{Database, Decode, Encode};

    #[test]
    fn it_works_with_a_real_database() -> BinaryMerkleTreeResult<()> {
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        let path = PathBuf::from("db");
        {
            let key = vec![0xAAu8];
            let mut values = vec![data.as_ref()];
            let mut tree = RocksTree::open(&path);
            let root;
            match tree.insert(None, &mut [&key], &mut values) {
                Ok(r) => root = r,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.remove(&root) {
                Ok(_) => {},
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => removed_retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
        }
        remove_dir_all(&path)?;
        assert_eq!(retrieved_value, vec![Some(data)]);
        assert_eq!(removed_retrieved_value, vec![None]);
        Ok(())
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

    struct RocksTree {
        tree: MerkleBIT<
            RocksDB,
            TreeBranch,
            TreeLeaf,
            TreeData,
            TreeNode,
            TreeHasher,
            TreeHashResult,
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

        pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
            self.tree.get(root_hash, keys)
        }

        pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&[u8]], values: &mut Vec<&Vec<u8>>) -> BinaryMerkleTreeResult<Vec<u8>> {
            self.tree.insert(previous_root, keys, values)
        }

        pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
            self.tree.remove(root_hash)
        }
    }
}
