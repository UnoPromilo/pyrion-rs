use tonic_prost_build::configure;

fn main() {
    configure()
        .compile_protos(&["proto/proto/v1/discovery.proto"], &["proto"])
        .unwrap();
}
