extern crate protoc_rust;

fn main() {
    println!("Building...");
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/serialization",
        input: &[
            "src/proto/tx.proto",
            "src/proto/blockHeader.proto",
            "src/proto/state.proto",
            "src/proto/block.proto",
            "src/proto/peer.proto",
            "src/proto/network.proto",
        ],
        includes: &["src/proto/"],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
