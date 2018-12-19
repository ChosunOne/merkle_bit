extern crate protoc_rust;

fn main() {
    build_proto();
}

fn build_proto() {
    println!("Building...");
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/serialization",
        input: &[
            "src/proto/state.proto",
        ],
        includes: &["src/proto/"],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
