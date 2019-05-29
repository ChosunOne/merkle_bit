#3.0.0
* Remove `use_rayon`.  Rayon doesn't seem well suited for the kind of parallelism required for the tree building process. 
* Change trait bounds on `MerkeBIT` to allow for auto derivation of `Sync + Send` if implemented in the underlying data types.  
* Removed unsafe internal sections of code. 
* Introduce new functions `MerkleBIT::generate_inclusion_proof` and `MerkleBIT::verify_inclusion_proof` which allow you to 
generate and verify inclusion proofs of items in a given root.  Verifying an inclusion proof requires the same type of tree
used to generate the inclusion proof.  The verifying tree may be empty.  
* Fix issue with missing call to batch_write for `RocksTree`.
#2.5.3
* Add `FxHash` support via `use_fx` feature.
* LTO has been enabled, giving a 10-15% performance boost across the board.
* `deconstruct` has been renamed `decompose`.
#2.5.2
* Add `SeaHash` support via `use_seahash` feature.
* Improve performance of `use_rayon`, though it is still slower than any other feature.
* Improve inlining support when LTO is disabled.
#2.5.1
* Futher performance improvements across the board, this time by around 20-30%.
* Added new unstable feature `use_rayon`.  It currently is much slower than any other feature, but will be receiving
attention in coming updates.  
# 2.5.0
* Keys must be explicity 32 bytes long, instead of slices
    * **NOTE:** This is a breaking change.  However, given that keys already had to be 32 bytes long, this change
        should be relatively painless.
* General performance improvements of about 30% across the board.
# 2.4.2
* Add error checking for root that is not 32 bytes long.
* Further major performance improvements.  Most cases see 50-70% reduction in time for insertions into a non-empty tree.
# 2.4.1
* Add error checking for keys that are not 32 bytes long.
# 2.4.0
* Require custom branch types to provide a key via ```get_key``` when requested
    * **NOTE:** This is a breaking change for custom data structures.  Usage of the default tree is not affected.
* Major performance upgrades, as much as 60% in some cases, though most cases see 20-30% improvements.
* Keys are now fixed to 32 bytes in size
    * **NOTE:** This is a breaking change.
# 2.3.1
* Simplify handling of errors within the crate.  ```Exception``` is used in place of ```Box<Error>```.
* Reduce the indirection in ```create_tree``` by compressing long pointer chains.  Results in approx 5% performance
improvements across the board.  
* Update ```serde-pickle```, ```ron```, ```openssl```, and ```rocksdb```.  
# 2.3.0
* Change return type of ```get``` to return a ```HashMap<&[u8], Option<ValueType>>``` instead of a ```Vec<Option<ValueType>>```.
This should resolve ambiguity of the return values when the input key list is not sorted.
    * **NOTE:** This is a breaking change. 
* Improve performance on inserting into non-empty trees for larger inserts.  There is a slight regression
in performance for smaller inserts, but the changes allowed for roughly 20% speed increases on inserts with 1000 entries or more.  
# 2.2.0
* Remove ```HashResultType``` from the tree in favor of using standard ```Vec<u8>```.
* Add benchmark for ```remove```.
* Improve performance for custom trees that don't store keys in branches.
* Require ```NodeType``` to have a ```NodeVariant``` on creation.
* Improve performance for default tree
# 2.1.3
* Improve performance for larger inserts
* Fix benchmarks to run on stable
# 2.1.2
* Allow ```Hashtree``` to accept any type implementing ```Encode``` and ```Decode```.
# 2.1.1
* Significant performance improvement for reads, as much as 30% over the last version.
* Insert performance has been improved by as much as 10% in most cases.
# 2.1.0
## Database Support
* The code has been restructured to make using some popular databases in addition to the existing serialization schemes (or with your own) much easier.  
Please see the ```rocks_tree.rs``` and ```rocksdb.rs``` files for an example on how to integrate your database with the existing tree.
* Add RocksDB support via the ```use_rocksdb``` feature 
## Structural Changes
* Many files have been split up into multiple other modules.  
* From this build on, the Git structure will change.  It will follow analogous to the current Rust structure, with a stable, beta, and nightly build. 
This should allow for more structured commits. 
* Many "unit" tests were really just integration tests, and as such have been moved to the proper area.  This has the bonus 
of allowing you to run the testing suite on more database types.
## Other Changes
* Improve overall performance by about 10% by removing a clone.
* Added ```use_hashbrown``` feature to use the hashbrown crate for HashTree.  This feature will be deprecated once hasbrown is included in the standard library and replaces the existing HashMap.
Until then, you can expect around a 10% boost to performance by using the hashbrown feature with the HashTree (and a smaller amount on other structures).
* Internal refactoring.  Would-be contributors should have a much easier time parsing the existing tree structure.
* **NOTE**:  There are a few minor breaking API changes in this release:
    * Some locations have changed with respect to the code restructuring.
    * ```HashTree::new``` now returns a ```Result```
    * ```HashTree::open``` has been added to fall in line with the API of the other databases.  It also returns a ```Result```.
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