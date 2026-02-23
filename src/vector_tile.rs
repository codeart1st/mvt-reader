#[cfg(feature = "protoc")]
include!(concat!(env!("OUT_DIR"), "/vector_tile.rs"));

#[cfg(not(feature = "protoc"))]
include!("generated/vector_tile.rs");
