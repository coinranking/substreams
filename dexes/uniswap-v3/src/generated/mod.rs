// @generated
pub mod google {
    // @@protoc_insertion_point(attribute:google.protobuf)
    pub mod protobuf {
        include!("google.protobuf.rs");
        // @@protoc_insertion_point(google.protobuf)
    }
}
pub mod sf {
    // @@protoc_insertion_point(attribute:sf.substreams)
    pub mod substreams {
        include!("sf.substreams.rs");
        // @@protoc_insertion_point(sf.substreams)
        pub mod entity {
            // @@protoc_insertion_point(attribute:sf.substreams.entity.v1)
            pub mod v1 {
                include!("sf.substreams.entity.v1.rs");
                // @@protoc_insertion_point(sf.substreams.entity.v1)
            }
        }
        pub mod index {
            // @@protoc_insertion_point(attribute:sf.substreams.index.v1)
            pub mod v1 {
                include!("sf.substreams.index.v1.rs");
                // @@protoc_insertion_point(sf.substreams.index.v1)
            }
        }
        pub mod rpc {
            // @@protoc_insertion_point(attribute:sf.substreams.rpc.v2)
            pub mod v2 {
                include!("sf.substreams.rpc.v2.rs");
                // @@protoc_insertion_point(sf.substreams.rpc.v2)
            }
        }
        pub mod sink {
            pub mod service {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.service.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.service.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.service.v1)
                }
            }
        }
        // @@protoc_insertion_point(attribute:sf.substreams.v1)
        pub mod v1 {
            include!("sf.substreams.v1.rs");
            // @@protoc_insertion_point(sf.substreams.v1)
        }
    }
}
pub mod uniswap {
    pub mod types {
        // @@protoc_insertion_point(attribute:uniswap.types.v1)
        pub mod v1 {
            include!("uniswap.types.v1.rs");
            // @@protoc_insertion_point(uniswap.types.v1)
        }
    }
    pub mod v3 {
        // @@protoc_insertion_point(attribute:uniswap.v3.mvp)
        pub mod mvp {
            include!("uniswap.v3.mvp.rs");
            // @@protoc_insertion_point(uniswap.v3.mvp)
        }
    }
}
