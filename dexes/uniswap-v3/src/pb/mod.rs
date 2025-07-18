pub mod uniswap {
    pub mod v3 {
        pub mod mvp {
            include!(concat!(env!("OUT_DIR"), "/uniswap.v3.mvp.rs"));
        }
    }
    pub mod types {
        pub mod v1 {
            include!("../generated/uniswap.types.v1.rs");
        }
    }
}
