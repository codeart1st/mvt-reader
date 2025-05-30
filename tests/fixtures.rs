mod common;

use common::{get_all_fixtures, TileContent, TileInfo};
use mvt_reader::geometry::{FlatCoordinateStorage, IdentityTransform};

use std::fs::{read, read_to_string};
use std::{io::Error, result::Result};

use mvt_reader::Reader;

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
        let layer_names_result = reader.get_layer_names();

        let layer_names = match layer_names_result {
          Ok(names) => names,
          Err(error) => {
            println!("{:?}", error);
            let info_str = read_to_string(info_file)?;
            let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;

            assert!(!info_json.validity.v1 && !info_json.validity.v2);
            println!("Failed correctly: {}", info_json.description);
            continue;
          }
        };

        if let Some(layers) = tile_json.layers {
          for layer in layers {
            match layer.name {
              Some(layer_name) => {
                assert!(layer_names.contains(&layer_name));
              }
              None => {
                let info_str = read_to_string(info_file)?;
                let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;

                assert!(!info_json.validity.v1 && !info_json.validity.v2);
                println!(
                  "Failed correctly, because missing layer name: {}",
                  info_json.description
                );
              }
            }
          }
        }

        for (i, _) in layer_names.iter().enumerate() {
          let features = reader.get_features_iter::<FlatCoordinateStorage, _>(i, IdentityTransform);
          match features {
            Some(features) => {
              println!("Parsed {} features", features.count());
            }
            None => {
              let info_str = read_to_string(info_file)?;
              let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;
              let mvt_file_path_string = mvt_file.to_str().unwrap();

              println!("Feature could not find");
              assert!(
                (!info_json.validity.v1 && !info_json.validity.v2)
                  || mvt_file_path_string.contains("016") // unknown geometry type
                  || mvt_file_path_string.contains("039") // unknown geometry type
              );
              println!(
                "Failed correctly, because incorrect features: {}",
                info_json.description
              );
            }
          }
        }

        println!("{:?}", layer_names);
      }
      Err(_) => {
        let info_str = read_to_string(info_file)?;
        let info_json: TileInfo = serde_json::from_str(info_str.as_str())?;

        assert!(!info_json.validity.v1 && !info_json.validity.v2);
        println!("Failed asdf correctly: {}", info_json.description);
      }
    }
  }
  Ok(())
}
