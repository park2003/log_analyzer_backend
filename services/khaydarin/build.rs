fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use tonic_prost_build as specified in build-dependencies
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../protos/savassan/v1/khaydarin.proto"],
            &["../../protos"],
        )?;

    // Rerun if proto files change
    println!("cargo:rerun-if-changed=../../protos/savassan/v1/khaydarin.proto");

    Ok(())
}
