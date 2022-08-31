fn main() {
    tonic_build::configure()
        .compile(&["proto/rustbase.proto"], &["proto"])
        .unwrap();
}
