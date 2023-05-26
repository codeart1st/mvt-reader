mod feature;
mod vector_tile;

use core::fmt;

use feature::Feature;
use geo_types::{Coord, GeometryCollection, LineString, Point, Polygon};
use prost::{bytes::Bytes, DecodeError, Message};
use vector_tile::Tile;

pub struct Reader {
  tile: Tile,
}

#[derive(Debug, Clone)]
pub struct VersionError {
  layer_name: String,

  version: u32,
}

impl fmt::Display for VersionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Vector tile version not supported for layer `{}` (found version: {})",
      self.layer_name, self.version
    )
  }
}

impl Reader {
  pub fn new(data: Vec<u8>) -> Result<Self, DecodeError> {
    Ok(Self {
      tile: Tile::decode(Bytes::from(data))?,
    })
  }

  pub fn get_layer_names(&self) -> Result<Vec<String>, VersionError> {
    let mut layer_names = Vec::with_capacity(self.tile.layers.len());
    for layer in self.tile.layers.iter() {
      match layer.version {
        1 | 2 => {
          layer_names.push(layer.name.clone());
        }
        _ => {
          return Err(VersionError {
            layer_name: layer.name.clone(),
            version: layer.version,
          })
        }
      }
    }
    Ok(layer_names)
  }

  pub fn get_features(&self, layer_index: usize) -> Option<Vec<Feature<GeometryCollection<f32>>>> {
    let layer = self.tile.layers.get(layer_index);
    match layer {
      Some(layer) => {
        let mut features = Vec::with_capacity(layer.features.len());
        for feature in layer.features.iter() {
          if let Some(geom_type) = feature.r#type {
            if let Some(geom_type) = vector_tile::tile::GeomType::from_i32(geom_type) {
              features.push(Feature {
                geometry: parse_geometry(&feature.geometry, geom_type),
                properties: Some(parse_tags(&feature.tags, &layer.keys, &layer.values)),
              });
            }
          }
        }
        Some(features)
      }
      None => None,
    }
  }
}

fn parse_tags(
  tags: &[u32],
  keys: &[String],
  values: &[vector_tile::tile::Value],
) -> std::collections::HashMap<String, String> {
  tags
    .chunks(2)
    .fold(std::collections::HashMap::new(), |mut acc, item| {
      acc.insert(
        (*keys.get(item[0] as usize).expect("item not found")).clone(),
        get_string_value((*values.get(item[1] as usize).expect("item not found")).clone()),
      );
      acc
    })
}

fn get_string_value(value: vector_tile::tile::Value) -> String {
  if value.string_value.is_some() {
    return value.string_value.unwrap();
  }
  if value.float_value.is_some() {
    return value.float_value.unwrap().to_string();
  }
  if value.double_value.is_some() {
    return value.double_value.unwrap().to_string();
  }
  if value.int_value.is_some() {
    return value.int_value.unwrap().to_string();
  }
  if value.uint_value.is_some() {
    return value.uint_value.unwrap().to_string();
  }
  if value.sint_value.is_some() {
    return value.sint_value.unwrap().to_string();
  }
  if value.bool_value.is_some() {
    return value.bool_value.unwrap().to_string();
  }
  String::new()
}

fn shoelace_formula(points: &[Point<f32>]) -> f32 {
  let mut area: f32 = 0.0;
  let n = points.len();
  let mut v1 = points[n - 1];
  for v2 in points.iter().take(n) {
    area += (v2.y() - v1.y()) * (v2.x() + v1.x());
    v1 = *v2;
  }
  area * 0.5
}

fn parse_geometry(
  geometry_data: &[u32],
  _geom_type: vector_tile::tile::GeomType,
) -> GeometryCollection<f32> {
  // worst case capacity to prevent reallocation. not needed to be exact.
  let mut coordinates = Vec::with_capacity(geometry_data.len());
  let mut rings: Vec<LineString<f32>> = Vec::new();
  let mut geometries = Vec::new();

  let mut cursor = [0, 0];
  let mut parameter_count: u32 = 0;
  let mut _id: u8 = 0;

  for (_, value) in geometry_data.iter().enumerate() {
    if parameter_count == 0 {
      let command_integer = value;
      _id = (command_integer & 0x7) as u8;
      match _id {
        1 | 2 => {
          // MoveTo | LineTo
          parameter_count = (command_integer >> 3) * 2; // 2-dimensional
        }
        7 => {
          // ClosePath
          coordinates.push(*coordinates.first().expect("invalid geometry"));

          let ring = LineString(coordinates);

          let area = shoelace_formula(&ring.clone().into_points());
          //info!("ClosePath with area: {} and coordinates {:?}", area, &ring);

          if area > 0.0 {
            // exterior ring
            //info!("exterior");
            if !rings.is_empty() {
              // finish previous geometry
              geometries.push(Polygon::new(rings[0].clone(), rings[1..].into()).into());
              rings = Vec::new();
            }
          } else {
            // interior ring
            //info!("interior");
          }
          rings.push(ring);
          // start a new sequence
          coordinates = Vec::new();
        }
        _ => (),
      }
    } else {
      let parameter_integer = value;
      let integer_value = ((parameter_integer >> 1) as i32) ^ -((parameter_integer & 1) as i32);
      if parameter_count % 2 == 0 {
        cursor[0] += integer_value;
      } else {
        cursor[1] += integer_value;
        /*match geom_type {
          vector_tile::tile::GeomType::Polygon => {
            info!("Polygon {} {}", cursor[0], cursor[1]);
          }
          vector_tile::tile::GeomType::Point => {
            info!("Point");
          }
          vector_tile::tile::GeomType::Linestring => {
            info!("Linestring");
          }
          _ => (),
        }*/
        coordinates.push(Coord {
          x: cursor[0] as f32,
          y: cursor[1] as f32,
        });
      }
      parameter_count -= 1;
    }
  }

  if !rings.is_empty() {
    // finish last geometry
    geometries.push(Polygon::new(rings[0].clone(), rings[1..].into()).into());
  }
  GeometryCollection(geometries)
}
