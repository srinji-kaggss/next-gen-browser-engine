// Build script: generate Rust protobuf bindings from proto/browser_state.proto.
// The generated module is included only when the `std` feature is enabled.
//
// Traceability: COH.proto.schema

fn main() {
    #[cfg(feature = "std")]
    {
        let proto_path = std::path::PathBuf::from("proto/browser_state.proto");
        if proto_path.exists() {
            prost_build::compile_protos(&[proto_path], &["proto/"]).expect("protobuf compile");
        }
    }
}
