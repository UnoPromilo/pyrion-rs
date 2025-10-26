use tonic_prost_build::configure;

fn main() {
    configure()
        .build_client(false)
        .compile_protos(
            &[
                "proto/pyrion/v1/discovery.proto",
                "proto/pyrion/v1/device_session.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
