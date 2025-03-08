use std::error::Error;
use std::fs::{DirEntry, read, read_dir};
use std::path::PathBuf;
use std::{io, result::Result};

use mvt_reader::error::TagsError;
use mvt_reader::{Reader, error::GeometryError};

#[test]
fn read_corrupted_geometry_fixture() -> Result<(), io::Error> {
  let bytes = [
    0x1a, 0x19, 0x12, 0x14, 0x10, 0x3d, 0x20, 0x11, 0x20, 0x28, 0x20, 0x28, 0x20, 0x20, 0x20, 0x20,
    0x20, 0x38, 0x20, 0x20, 0x18, 0x03, 0x40, 0x11, 0x60, 0xef, 0x23,
  ];
  let reader_result = Reader::new(bytes.to_vec());
  match reader_result {
    Ok(reader) => {
      let features = reader.get_features(0);
      match features {
        Ok(_) => {
          panic!("Parsing should have failed")
        }
        Err(error) => match error.source() {
          Some(error) if error.is::<GeometryError>() => {
            println!("Expected error: {}", error);
          }
          _ => {
            panic!("Unexpected error: {}", error)
          }
        },
      }
    }
    Err(_) => {
      panic!("Parsing failed unexpectedly")
    }
  }
  Ok(())
}

#[test]
fn read_corrupted_tags_fixture() -> Result<(), io::Error> {
  let bytes = [
    0x1a, 0x19, 0x12, 0x14, 0x10, 0x0, 0x10, 0x0, 0x20, 0x20, 0x10, 0x28, 0x20, 0x28, 0xe0, 0xdf,
    0x20, 0x20, 0x20, 0x32, 0x20, 0x20, 0x18, 0x1, 0x60, 0xef, 0x1b,
  ];
  let reader_result = Reader::new(bytes.to_vec());
  match reader_result {
    Ok(reader) => {
      let features = reader.get_features(0);
      match features {
        Ok(_) => {
          panic!("Parsing should have failed")
        }
        Err(error) => match error.source() {
          Some(error) if error.is::<TagsError>() => {
            println!("Expected error: {}", error);
          }
          _ => {
            panic!("Unexpected error: {}", error)
          }
        },
      }
    }
    Err(_) => {
      panic!("Parsing failed unexpectedly")
    }
  }
  Ok(())
}

#[test]
fn read_all_fixtures() -> Result<(), io::Error> {
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
          let features = reader.get_features(i);
          assert!(!features.unwrap().is_empty());
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

pub fn get_all_real_world_fixtures() -> Result<Vec<PathBuf>, io::Error> {
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
