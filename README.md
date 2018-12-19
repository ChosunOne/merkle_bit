# Merkle Binary Indexed Tree (Merkle-BIT)
This tree structure is a binary merkle tree with branch compression via split indexes.  See [here](https://medium.com/@niallmoore22/binary-merkle-trie-aad76f422983) for a basic explanation of its purpose.

## Basic Usage
```rust
    extern crate starling;
    use starling::common::merkle_bit::MerkleBIT;
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
                             
        // Keys must by slices of u8 arrays or vectors
        let key: Vec<u8> = vec![0x00u8, 0x81u8, 0xA3u8];
        
        // An example value created from ValueType.  
        let value: ValueType = ValueType::new("Some value");
        
        // You can specify a previous root to add to, in this case there is no previous root
        let root: Vec<u8> = mbit.insert(None, &[key.as_ref()], &[value.as_ref()])?;
        
        // Retrieving the inserted value
        let inserted_values: Vec<Option<ValueType>> = mbit.get(root.as_ref(), &[key.as_ref()])?;
        
        // Removing a tree root
        mbit.remove(root.as_ref())?;
        
    }
```