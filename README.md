[![GitHub release](https://img.shields.io/github/release/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/releases) [![Crates.io](https://img.shields.io/crates/v/starling.svg)](https://crates.io/crates/starling) [![Crates.io](https://img.shields.io/crates/l/starling.svg)](https://github.com/ChosunOne/merkle_bit/blob/stable/LICENSE-APACHE) [![GitHub last commit](https://img.shields.io/github/last-commit/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/commits/stable) [![dependency status](https://deps.rs/repo/github/ChosunOne/merkle_bit/status.svg)](https://deps.rs/repo/github/ChosunOne/merkle_bit)
[![Travis (.com)](https://img.shields.io/travis/com/ChosunOne/merkle_bit.svg)](https://travis-ci.com/ChosunOne/merkle_bit/builds) [![GitHub issues](https://img.shields.io/github/issues-raw/ChosunOne/merkle_bit.svg)](https://github.com/ChosunOne/merkle_bit/issues) 
[![Codecov branch](https://img.shields.io/codecov/c/github/ChosunOne/merkle_bit/stable.svg)](https://codecov.io/gh/ChosunOne/merkle_bit)  ![Crates.io](https://img.shields.io/crates/d/starling.svg) [![Gitter](https://img.shields.io/gitter/room/merkle_bit/merkle_bit.svg)](https://gitter.im/merkle_bit/community) [![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://paypal.me/ChosunOne?locale.x=en_US)
# Merkle Binary Indexed Tree (Merkle-BIT)
This tree structure is a binary merkle tree with branch compression via split indexes.  This structure can be used to store multiple versions of tree state without any duplication of the stored data, either in memory or on disk.  See [here](https://ethereum.stackexchange.com/questions/15288/ethereum-merkle-tree-explanation) and [here](https://medium.com/@niallmoore22/binary-merkle-trie-aad76f422983) for a basic explanation of its purpose.

## Basic Usage
To quickly get started and get a feel for the Merkle-BIT, you can use the already implemented HashTree structure.

```rust
    use std::error::Error;
    use starling::hash_tree::HashTree;
    
    fn main() -> Result<Ok(), Error> {
        let tree = HashTree::new(8)?;
        
        // Keys must be of fixed size
        let mut key: Array<32> = [0xFF; 32].into();
        
        // Value to be put into the tree
        let value: Vec<u8> = vec![0xDDu8];
        
        // Inserting an element changes the root node
        let root = tree.insert(None, &mut [&key], &[value])?;
        
        let retrieved_value = tree.get(&root, &mut [&key])?;
        
        // Removing a root only deletes elements that are referenced only by that root
        tree.remove(&root)?;
        Ok(())
    }
```

This structure can be used for small amounts of data, but all the data in the tree will persist in memory unless explicitly pruned.

For larger numbers of items to store in the tree, it is recommended to connect the structure to a database by implementing the 
`Database` trait for your database.  This structure will also take advantage of batch writes if your database supports it.  

## Benchmarks

Below are the benchmarks when using ```starling``` on an in-memory database on a reasonably fast machine:

| Operation   |      Num. Entries      |  Is Tree Empty? | Measured Benchmark |
|----------|-------------:|------:|-----:|
| insertion |  1 | yes | 0.407μs |
| insertion |  10 | yes | 5.136μs |
| insertion | 100 | yes | 46.796μs |
| insertion | 1000 | yes | 480.060μs |
| insertion | 10000 | yes | 7,219.300μs |
| insertion | 1 | no | 6.315μs |
| insertion | 10 | no | 19.400μs |
| insertion | 100 | no | 149.710μs |
| insertion | 1000 | no | 1,517.700μs |
| insertion | 10000 | no | 15,043.000μs |
| retrieval | 4096 | no | 2,889.100μs |
| retrieval | 10000 | no | 9,437.100μs |
| removal | 4096 | no | 0.070μs |
| removal | 10000 | no | 0.071μs |

## Features
Starling supports a number of serialization and hashing schemes for use in the tree, which should be selected based on 
your performance and application needs.

Currently integrated serialization schemes include:
* `bincode`
* `serde-json`
* `serde-cbor`
* `serde-yaml`
* `serde-pickle`
* `ron`

It should be noted that any serialization scheme will work with starling, provided you implement the ```Encode``` and ```Decode``` traits for the node types.

Currently integrated tree hashing schemes include:
* `Blake2b` via `blake2_rfc`
* `Groestl` via `groestl`
* `SHA2` via `openssl`
* `SHA3` via `tiny-keccak`
* `Keccak` via `tiny-keccak`
* `SeaHash` via `seahash`
* `FxHash` via `fxhash`
* and most updated hashes from [RustCrypto](https://github.com/RustCrypto/hashes)

You may also use the default Rust hasher, or implement the ```Hasher``` trait for your own hashing scheme (unless using a hash from 
RustCrypto, then you will want to enable the `digest` feature, which implements `Hasher` for `Digest`).

You can also use RocksDB to handle storing and loading from disk.
You can use the ```RocksTree``` with a serialization scheme via the ```--features="rocksdb bincode"``` command line flags 
or by enabling the features in your Cargo.toml manifest.

Some enabled features must be used in combination, or you must implement the required traits yourself (E.g. using the 
```rocksdb``` feature alone will generate a compiler error, you must also select a serialization scheme, such as ```bincode``` or implement it for your data).

Finally, you can take advantage of the ```hashbrown``` to use the ```hasbrown``` crate instead of the standard library ```HashMap```.

## Full Customization

To use the full power of the Merkle-BIT structure, you should customize the structures stored in the tree to match your needs.  

If you provide your own implementation of the traits for each component of the tree structure, the tree can utilize them over the default implementation.
```rust
    use starling::merkle_bit::MerkleBIT;
    use starling::Array;
    use std::path::Path;
    use std::error::Error;
    
    fn main() -> Result<Ok, Error> {
        // A path to a database to be opened
        let path = Path::new("some path");
        
        // Your own database library
        let db = YourDB::open(&path);
        
        // These type annotations are required to specialize the Merkle BIT
        // Check the documentation for the required trait bounds for each of these types.
        let mbit = MerkleBIT<DatabaseType, 
                             BranchType, 
                             LeafType, 
                             DataType, 
                             NodeType, 
                             HasherType, 
                             ValueType,
                             32>::from_db(db, depth);
                             
        // Keys must be of fixed size
        let key: Array<32> = [0xFF; 32].into();
        
        // An example value created from ValueType.  
        let value: ValueType = ValueType::new("Some value");
        
        // You can specify a previous root to add to, in this case there is no previous root
        let root: Array<32> = mbit.insert(None, &mut [key], &[value])?;

        // Every time an element is added or removed a new root is created.
        let new_key: Array<32> = [0xEE; 32].into();
        let new_value: ValueType = ValueType::new("Some new value");
        let new_root: Array<32> = mbit.insert(&root, &mut [key], &[value])?;
        
        // Retrieving the inserted value
        let inserted_values: HashMap<&Array<32>, Option<ValueType>> = mbit.get(&root, &mut [key])?;

        // You must ensure that the root you supply matches a root where the key existed when retrieving items
        // This line will fail to find the `new_value`
        let empty_map = mbit.get(&root, &mut [new_key])?;

        // This line will succeed in finding values for both `key` and `new_key`
        let inhabited_map = mbit.get(&new_root, &mut [key, new_key])?;

        
        // Removing a tree root
        mbit.remove(&root)?;

        // This line will fail to find a value for `key` but will succeed in finding the value for `new_key`
        let partially_inhabited_map = mbit.get(&new_root, &mut [key, new_key])?;
        Ok(())
    }
```

## Verification

The `MerkleBIT` also supports generating and verifying merkle inclusion proofs, and may be used like below:
```rust
    use starling::hash_tree::HashTree;
    use std::error::Error;
    
    fn main() -> Result<Ok, Error> {
        let tree = HashTree::new(8)?;
        
        let mut key: Array<32> = [0xFF; 32].into();
        let value: Vec<u8> = vec![0xDDu8];
        
        let root: Array<32> = tree.insert(None, &mut [&key], &[value])?;
        
        // An inclusion proof that proves membership of a key in the tree
        let proof: Vec<(Array<32>, bool)> = tree.generate_inclusion_proof(&root, key)?;
        
        // If the proof is valid, it will return Ok(())
        HashTree::verify_inclusion_proof(&root, key, &value, &proof)?;
        Ok(())
    }
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Reporting
The project is currently undergoing rapid development and it should be noted that minor releases may include breaking changes
to the API.  These changes will be noted in the Changelog of each release, but if we broke something or forgot to mention 
such a change, please [file an issue](https://github.com/ChosunOne/merkle_bit/issues/new/choose) or 
[submit a pull request](https://github.com/ChosunOne/merkle_bit/compare) and we will review it at our earliest convenience.

## Support
Do you use this crate and would like to ensure continued support?  Please consider supporting me via Github Sponsors at 
[my sponsor page](https://github.com/sponsors/ChosunOne).

#### Acknowledgments
Special thanks to Niall Moore and Owen Delahoy for assistance with the early phases of this project. 