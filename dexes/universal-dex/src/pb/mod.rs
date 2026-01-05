pub mod evm {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/evm.common.v1.rs"));
        }
    }
}
