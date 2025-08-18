use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the project root directory (two levels up from services/flyte-integration)
    let proto_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("protos");

    // Compile the execution.proto file
    tonic_prost_build::configure()
        .build_server(true) // Generate server code for ExecutionService
        .build_client(true) // Generate client code for calling other services
        // Don't specify out_dir - use default OUT_DIR for proper tonic::include_proto! support
        .compile_protos(
            &[proto_root.join("savassan/v1/execution.proto")],
            &[proto_root],
        )?;

    // Tell Cargo to recompile if the proto file changes
    println!("cargo:rerun-if-changed=../../protos/savassan/v1/execution.proto");

    Ok(())
}
