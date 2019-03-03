[![GitHub release](https://img.shields.io/github/release/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/releases) [![Crates.io](https://img.shields.io/crates/v/starling.svg)](https://crates.io/crates/starling) [![Crates.io](https://img.shields.io/crates/l/starling.svg)](https://github.com/ChosunOne/merkle_bit/blob/master/LICENSE) [![GitHub last commit](https://img.shields.io/github/last-commit/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/commits/master)  [![Travis (.com)](https://img.shields.io/travis/com/ChosunOne/merkle_bit.svg)](https://travis-ci.com/ChosunOne/merkle_bit/builds) [![GitHub issues](https://img.shields.io/github/issues-raw/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/issues) ![Crates.io](https://img.shields.io/crates/d/starling.svg) [![Gitter](https://img.shields.io/gitter/room/merkle_bit/merkle_bit.svg)](https://gitter.im/merkle_bit/community) [![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://paypal.me/ChosunOne?locale.x=en_US)
# Merkle Binary Indexed Tree (Merkle-BIT)
This tree structure is a binary merkle tree with branch compression via split indexes.  See [here](https://medium.com/@niallmoore22/binary-merkle-trie-aad76f422983) for a basic explanation of its purpose.

## Basic Usage
To quickly get started and get a feel for the Merkle-BIT, you can use the already implemented HashTree structure.

```rust
    extern crate starling;
    use starling::tree::HashTree;
    
    fn main() {
        let tree = HashTree::new(8);
        
        // Keys must be slices of u8 arrays or vectors
        let mut key: Vec<u8> = vec![0x00u8, 0x81u8, 0xA3u8];
        
        // The HashTree only deals with byte vectors,
        // you must serialize your object prior to putting it into the HashTree
        let value: Vec<u8> = vec![0xDDu8];
        
        // Inserting an element changes the root node
        let root = tree.insert(None, &mut [key.as_ref()], &mut [value.as_ref()]).unwrap();
        
        let retrieved_value = tree.get(root.as_ref(), &mut [key.as_ref()]).unwrap();
        
        // Removing a root only deletes elements that are referenced only by that root
        tree.remove(root.as_ref()).unwrap();
    }
```

This structure can be used for small amounts of data, but all the data in the tree will persist in memory unless explicitly pruned.

For larger numbers of items to store in the tree, it is recommended to connect the structure to a database by implementing the 
Database trait for your database.  This structure will also take advantage of batch writes if your database supports it.  

You can take advantage of many of serde's serializing and deserializing data schemes 
prior to putting it into a database via the features of this crate.  Additionally, there are several supported hashing schemes you can use via features in the crate.

To use the full power of the Merkle-BIT structure, you should customize the structures stored in the tree to match your needs.  
```rust
    extern crate starling;
    use starling::merkle_bit::MerkleBIT;
    use std::path::PathBuf;
    
    fn main() {
        // A path to a database to be opened
        let path = PathBuf::new("some path");
        
        // These type annotations are required to specialize the Merkle BIT
        // Check the documentation for the required trait bounds for each of these types.
        let mbit = MerkleBIT<DatabaseType, 
                             BranchType, 
                             LeafType, 
                             DataType, 
                             NodeType, 
                             HasherType, 
                             HashResultType, 
                             ValueType>::new(path, 8);
                             
        // Keys must be slices of u8 arrays or vectors
        let key: Vec<u8> = vec![0x00u8, 0x81u8, 0xA3u8];
        
        // An example value created from ValueType.  
        let value: ValueType = ValueType::new("Some value");
        
        // You can specify a previous root to add to, in this case there is no previous root
        let root: Vec<u8> = mbit.insert(None, &mut [key.as_ref()], &mut [value.as_ref()])?;
        
        // Retrieving the inserted value
        let inserted_values: Vec<Option<ValueType>> = mbit.get(root.as_ref(), &mut [key.as_ref()])?;
        
        // Removing a tree root
        mbit.remove(root.as_ref())?;
        
    }
```