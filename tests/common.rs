use serde::Deserialize;
use std::fs::read_dir;
use std::path::PathBuf;
use std::{io::Error, result::Result};

type Fixture = (PathBuf, PathBuf, PathBuf);

pub fn get_all_fixtures() -> Result<Vec<Fixture>, Error> {
  let mut result = Vec::new();

  let mut entries: Vec<_> = read_dir("mvt-fixtures/fixtures")?
    .map(|entry| entry.unwrap())
    .collect();

  entries.sort_by(|a, b| {
    let name_a = a.file_name();
    let name_b = b.file_name();

    name_a.cmp(&name_b)
  });

  for entry in entries {
    let mvt_file = entry.path().join("tile.mvt");
    let tile_file = entry.path().join("tile.json");
    let info_file = entry.path().join("info.json");

    result.push((mvt_file, tile_file, info_file));
  }
  Ok(result)
}

#[derive(Deserialize)]
pub struct Validity {
  pub v1: bool,
  pub v2: bool,
}

#[derive(Deserialize)]
pub struct TileInfo {
  pub validity: Validity,
}

#[derive(Deserialize)]
pub struct Layer {
  pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct TileContent {
  pub layers: Option<Vec<Layer>>,
}
