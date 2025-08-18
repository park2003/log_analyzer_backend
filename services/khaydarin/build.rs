fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the path to the protos directory (two levels up from service)

    // Configure tonic_build
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        // Don't specify out_dir - use default OUT_DIR for proper tonic::include_proto! support
        .compile_protos(
            &["savassan/v1/khaydarin.proto", "savassan/v1/savassan.proto"],
            &["../../protos"],
        )?;

    // Rerun if proto files change
    println!("cargo:rerun-if-changed=../../protos/savassan/v1/khaydarin.proto");
    println!("cargo:rerun-if-changed=../../protos/savassan/v1/savassan.proto");

    Ok(())
}
