extern crate protoc_rust_no_elision;

fn main() {
    build_proto();
}

fn build_proto() {
    println!("Building...");
    protoc_rust_no_elision::run(protoc_rust_no_elision::Args {
        out_dir: "src/serialization",
        input: &[
            "src/proto/state.proto",
        ],
        includes: &["src/proto/"],
        customize: protoc_rust_no_elision::Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
