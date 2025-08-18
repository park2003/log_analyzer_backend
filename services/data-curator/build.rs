fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf files
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../protos/savassan/v1/data_curator.proto"],
            &["../../protos"],
        )?;

    println!("cargo:rerun-if-changed=../../protos/savassan/v1/data_curator.proto");

    Ok(())
}
