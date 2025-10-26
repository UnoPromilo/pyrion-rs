pub mod pyrion {
    pub mod v1 {
        pub mod discovery {
            tonic::include_proto!("pyrion.v1.discovery");
        }
        pub mod interface {
            tonic::include_proto!("pyrion.v1.interface");
        }
        pub mod controller_message {
            tonic::include_proto!("pyrion.v1.controller_message");
        }
        pub mod device_message {
            tonic::include_proto!("pyrion.v1.device_message");
        }
        pub mod device_session {
            tonic::include_proto!("pyrion.v1.device_session");
        }
    }
}
