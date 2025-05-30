use std::fs::{read, read_dir, DirEntry};
use std::path::PathBuf;
use std::{io::Error, result::Result};

use mvt_reader::{
  geometry::{FlatCoordinateStorage, IdentityTransform},
  Reader,
};

#[test]
fn read_all_fixtures() -> Result<(), Error> {
  for mvt_file in get_all_real_world_fixtures()?.iter() {
    if !mvt_file.extension().unwrap().eq_ignore_ascii_case("mvt") {
      println!("Skipped file {:?}", mvt_file);
      continue;
    }
    println!("Read {:?}", mvt_file);
    let bytes = read(mvt_file)?;
    let reader_result = Reader::new(bytes.to_vec());
    match reader_result {
      Ok(reader) => {
        let layer_names = match reader.get_layer_names() {
          Ok(layer_names) => layer_names,
          Err(error) => {
            panic!("{}", error);
          }
        };
        for (i, _) in layer_names.iter().enumerate() {
          let features = reader.get_features_iter::<FlatCoordinateStorage, _>(i, IdentityTransform);
          assert!(features.unwrap().count() > 0);
        }
        println!("found layer names: {:?}", layer_names);
      }
      Err(_) => {
        panic!("Parsing failed unexpectedly")
      }
    }
  }
  Ok(())
}

pub fn get_sorted_dir_entries(path: &str) -> std::io::Result<Vec<DirEntry>> {
  let mut entries: Vec<_> = read_dir(path)?.map(|entry| entry.unwrap()).collect();

  entries.sort_by(|a, b| {
    let name_a = a.file_name();
    let name_b = b.file_name();

    name_a.cmp(&name_b)
  });

  Ok(entries)
}

pub fn get_all_real_world_fixtures() -> Result<Vec<PathBuf>, Error> {
  let mut result = Vec::new();

  let countries = get_sorted_dir_entries("mvt-fixtures/real-world")?;
  for country in countries {
    let fixtures = get_sorted_dir_entries(country.path().to_str().unwrap())?;
    for fixture in fixtures {
      result.push(fixture.path());
    }
  }
  Ok(result)
}
