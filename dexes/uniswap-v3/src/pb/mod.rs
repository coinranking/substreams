pub mod dex {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/dex.common.v1.rs"));
        }
    }
}

pub mod uniswap {
    pub mod types {
        #[allow(clippy::enum_variant_names)]
        pub mod v1 {
            include!("../generated/uniswap.types.v1.rs");
        }
    }
}
