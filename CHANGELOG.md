# 1.1.1
* Update to 2018 edition of Rust
* Minor code style changes
* Update dev-dependencies

# 1.1.0
* Removed Encode and Decode trait bounds for Node type
* Added usable implementation for the Merkle-BIT with a HashMap backend (HashTree)  
* Added support for storing branch keys to avoid extra DB lookups
* Renamed some traits and enums to better describe their purpose