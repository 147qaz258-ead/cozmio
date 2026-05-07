fn main() {
    println!("cargo:rerun-if-changed=src/proto/relay.proto");
    println!("cargo:rerun-if-changed=src/proto/mod.rs");
}
