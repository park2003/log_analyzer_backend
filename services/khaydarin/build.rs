fn main() {
    // TODO: Configure tonic_build for proto compilation
    // The exact API depends on the tonic_build version
    println!("cargo:rerun-if-changed=../../protos/savassan/v1/khaydarin.proto");
}