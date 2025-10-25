use tonic_prost_build::configure;

fn main() {
    configure()
        .build_client(false)
        .compile_protos(&["proto/proto/v1/discovery.proto"], &["proto"])
        .unwrap();
}
