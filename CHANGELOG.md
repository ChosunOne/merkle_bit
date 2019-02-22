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