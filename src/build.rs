fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/rustbase.proto"], &["proto"])
        .unwrap();
}
