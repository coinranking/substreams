pub mod dex {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/dex.common.v1.rs"));
        }
    }
}
