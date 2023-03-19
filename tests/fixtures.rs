mod common;

use common::{get_all_fixtures, TileContent, TileInfo};
use mvt_reader::Reader;
use std::fs::{read, read_to_string};
use std::{io::Error, result::Result};

#[test]
fn read_all_fixtures() -> Result<(), Error> {
  for (mvt_file, tile_file, info_file) in get_all_fixtures()?.iter() {
    println!("Read {:?}", mvt_file);
    let bytes = read(mvt_file)?;
    let reader_result = Reader::new(bytes.to_vec());
    match reader_result {
      Ok(reader) => {
        let tile_str = read_to_string(tile_file)?;
        let tile_json: TileContent = serde_json::from_str(tile_str.as_str())?;
        let layer_names = reader.get_layer_names();

        if let Some(layers) = tile_json.layers {
          for layer in layers {
            match layer.name {
              Some(layer_name) => {
                assert!(layer_names.contains(&layer_name));
              }
              None => {
                let info_str = read_to_string(info_file)?;
                let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;

                // TODO: get_layer_names should fail for this szenario
                assert!(!info_json.validity.v1 && !info_json.validity.v2);
                println!("Failed correctly, because missing layer name");
              }
            }
          }
        }

        println!("{:?}", layer_names);
      }
      Err(_) => {
        let info_str = read_to_string(info_file)?;
        let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;

        assert!(!info_json.validity.v1 && !info_json.validity.v2);
        println!("Failed correctly");
      }
    }
  }
  Ok(())
}
