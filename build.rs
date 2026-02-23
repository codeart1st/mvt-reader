use std::io::Result;

fn main() -> Result<()> {
  #[cfg(any(feature = "protoc", feature = "protoc-generated"))]
  {
    let mut prost_build = prost_build::Config::new();
    prost_build.btree_map(["."]);
    #[cfg(feature = "protoc-generated")]
    prost_build.out_dir("src/generated");
    prost_build.compile_protos(&["vector-tile-spec/2.1/vector_tile.proto"], &["."])?;
  }
  Ok(())
}
