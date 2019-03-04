# 2.1.0
* Added ```use_hashbrown``` feature to use the hashbrown crate for HashTree.  This feature will be deprecated once hasbrown is included in the standard library and replaces the existing HashMap.
Until then, you can expect around a 10% boost to performance by using the hashbrown feature.
* Internal refactoring.  Would-be contributors should have a much easier time parsing the existing tree structure.
* **NOTE**:  There are a few minor breaking API changes in this release, but only if you are implementing your own tree structure.  
Only the location of some structures have changed, not the function signatures.
# 2.0.2
* Minor internal optimization
# 2.0.0
* Separate serde from ```default_tree``` feature, now use ```use_serde``` to take advantage of 
serde for serialization, though a number of serde schemes are implemented as their own features (see below).
* Separate bincode from ```default_tree```.  To use bincode with the default tree, you only need to use the "use_bincode" feature
ex. ```cargo build --features "use_bincode"```
## New serialization schemes
* Add JSON support through ```use_json``` feature
* Add CBOR support through ```use_cbor``` feature
* Add YAML support through ```use_yaml``` feature
* Add Pickle support through ```use_pickle``` feature
* Add RON support through ```use_ron``` feature
## New hashing schemes
* You can now use different hashing schemes with the different serialization features.
* Add Blake2b support through ```use_blake2b``` feature
* Add Groestl support through ```use_groestl``` feature (note: Groestl is much slower compared to the other hashing algorithms)
* Add SHA-2 (SHA256) support through ```use_sha2``` feature
* Add SHA-3 support through ```use_sha3``` feature
* Add Keccak256 support through ```use_keccak``` feature
## Bug Fixes
* Fixed issue with getting values when supplied keys were not all in the tree
* Fixed issue when using stored split index values on inserts.
* Inputs to get and insert no longer need to be sorted (sorting is done internally)
## Development Improvements
* Added benchmarking via ```cargo bench```
* Added fuzzing via ```cargo +nightly fuzz <fuzz_target_name>```.  Requires installation of ```cargo-fuzz``` and ```nightly``` toolchain.
# 1.2.1
* Add serde support for default tree implementation
* You can now use the "default_tree" feature for a tree structure relying on serde and
bincode for serialization prior to entering a database. This significantly reduces the boilerplate code needed to connect the tree to a
database.
* Added integration test with RocksDB, to run the test you may run  
```cargo test --features="default_tree"```

# 1.1.1
* Update to 2018 edition of Rust
* Minor code style changes
* Update dev-dependencies

# 1.1.0
* Removed Encode and Decode trait bounds for Node type
* Added usable implementation for the Merkle-BIT with a HashMap backend (HashTree)  
* Added support for storing branch keys to avoid extra DB lookups
* Renamed some traits and enums to better describe their purpose