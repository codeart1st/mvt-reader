use std::io::Result;

fn main() -> Result<()> {
  let mut prost_build = prost_build::Config::new();
  prost_build.btree_map(["."]);
  prost_build.compile_protos(&["vector-tile-spec/2.1/vector_tile.proto"], &["."])?;
  Ok(())
}
