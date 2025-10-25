use tonic_prost_build::configure;

fn main() {
    configure()
        .compile_protos(&["proto/discovery.proto"], &["proto"])
        .unwrap();
}
