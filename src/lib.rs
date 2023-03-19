mod vector_tile;

use prost::{bytes::Bytes, DecodeError, Message};
use vector_tile::Tile;

pub struct Reader {
  tile: Tile,
}

impl Reader {
  pub fn new(data: Vec<u8>) -> Result<Self, DecodeError> {
    Ok(Self {
      tile: Tile::decode(Bytes::from(data))?,
    })
  }

  pub fn get_layer_names(&self) -> Vec<String> {
    let mut layer_names = Vec::with_capacity(self.tile.layers.len());
    for layer in self.tile.layers.iter() {
      match layer.version {
        1 | 2 => {
          layer_names.push(layer.name.clone());
        }
        _ => {
          println!(
            "Vector tile version not supported for layer `{}` (found version: {})",
            layer.name, layer.version
          );
        }
      }
    }
    layer_names
  }
}
