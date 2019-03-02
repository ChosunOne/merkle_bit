# 2.0.0
* Separate serde from "default_tree" feature, now use "use_serde" to take advantage of 
serde for serialization, though a number of serde schemes are implemented as their own features (see below).
* Separate bincode from "default_tree".  To use bincode with the default tree, you only need to use the "use_bincode" feature
ex. ```cargo build --features "use_bincode"```
* Add JSON support through "use_json" feature
* Add CBOR support through "use_cbor" feature
* Add YAML support through "use_yaml" feature
* Add Pickle support through "use_pickle" feature
* Add RON support through "use_ron" feature
* Fixed issue with getting values when supplied keys were not all in the tree
* Inputs to get and insert no longer need to be sorted (sorting is done internally)
* Fixed issue when using stored split index values on inserts.

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