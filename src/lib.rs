pub mod error;
pub mod feature;

mod vector_tile;

use feature::Feature;
use geo_types::{
  Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use prost::{bytes::Bytes, DecodeError, Message};
use vector_tile::{tile::GeomType, Tile};

const DIMENSION: u32 = 2;

pub struct Reader {
  tile: Tile,
}

impl Reader {
  pub fn new(data: Vec<u8>) -> Result<Self, DecodeError> {
    Ok(Self {
      tile: Tile::decode(Bytes::from(data))?,
    })
  }

  pub fn get_layer_names(&self) -> Result<Vec<String>, error::ParserError> {
    let mut layer_names = Vec::with_capacity(self.tile.layers.len());
    for layer in self.tile.layers.iter() {
      match layer.version {
        1 | 2 => {
          layer_names.push(layer.name.clone());
        }
        _ => {
          return Err(error::ParserError::new(error::VersionError::new(
            layer.name.clone(),
            layer.version,
          )))
        }
      }
    }
    Ok(layer_names)
  }

  pub fn get_features(&self, layer_index: usize) -> Result<Vec<Feature>, error::ParserError> {
    let layer = self.tile.layers.get(layer_index);
    match layer {
      Some(layer) => {
        let mut features = Vec::with_capacity(layer.features.len());
        for feature in layer.features.iter() {
          if let Some(geom_type) = feature.r#type {
            if let Some(geom_type) = GeomType::from_i32(geom_type) {
              let parsed_geometry = match parse_geometry(&feature.geometry, geom_type) {
                Ok(parsed_geometry) => parsed_geometry,
                Err(error) => {
                  return Err(error);
                }
              };

              let parsed_tags = match parse_tags(&feature.tags, &layer.keys, &layer.values) {
                Ok(parsed_tags) => parsed_tags,
                Err(error) => {
                  return Err(error);
                }
              };

              features.push(Feature {
                geometry: parsed_geometry,
                properties: Some(parsed_tags),
              });
            }
          }
        }
        Ok(features)
      }
      None => Ok(vec![]),
    }
  }
}

fn parse_tags(
  tags: &[u32],
  keys: &[String],
  values: &[vector_tile::tile::Value],
) -> Result<std::collections::HashMap<String, String>, error::ParserError> {
  let mut result = std::collections::HashMap::new();
  for item in tags.chunks(2) {
    if item.len() != 2
      || item[0] > keys.len().try_into().unwrap()
      || item[1] > values.len().try_into().unwrap()
    {
      return Err(error::ParserError::new(error::TagsError::new()));
    }
    result.insert(
      (*keys.get(item[0] as usize).expect("item not found")).clone(),
      get_string_value((*values.get(item[1] as usize).expect("item not found")).clone()),
    );
  }
  Ok(result)
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
  geom_type: GeomType,
) -> Result<Geometry<f32>, error::ParserError> {
  if geom_type == GeomType::Unknown {
    return Err(error::ParserError::new(error::GeomtryError::new()));
  }

  // worst case capacity to prevent reallocation. not needed to be exact.
  let mut coordinates: Vec<Coord<f32>> = Vec::with_capacity(geometry_data.len());
  let mut polygons: Vec<Polygon<f32>> = Vec::new();
  let mut linestrings: Vec<LineString<f32>> = Vec::new();

  let mut cursor: [i32; 2] = [0, 0];
  let mut parameter_count: u32 = 0;

  for (_, value) in geometry_data.iter().enumerate() {
    if parameter_count == 0 {
      let command_integer = value;
      let id = (command_integer & 0x7) as u8;
      match id {
        1 => {
          // MoveTo
          parameter_count = (command_integer >> 3) * DIMENSION;
          if geom_type == GeomType::Linestring && !coordinates.is_empty() {
            linestrings.push(LineString::new(coordinates));
            // start with a new linestring
            coordinates = Vec::with_capacity(geometry_data.len());
          }
        }
        2 => {
          // LineTo
          parameter_count = (command_integer >> 3) * DIMENSION;
        }
        7 => {
          // ClosePath
          let first_coordinate = match coordinates.first() {
            Some(coord) => coord.to_owned(),
            None => {
              return Err(error::ParserError::new(error::GeomtryError::new()));
            }
          };
          coordinates.push(first_coordinate);

          let ring = LineString::new(coordinates);

          let area = shoelace_formula(&ring.clone().into_points());

          if area > 0.0 {
            // exterior ring
            if !linestrings.is_empty() {
              // finish previous geometry
              polygons.push(Polygon::new(
                linestrings[0].clone(),
                linestrings[1..].into(),
              ));
              linestrings = Vec::new();
            }
          }

          linestrings.push(ring);
          // start a new sequence
          coordinates = Vec::with_capacity(geometry_data.len());
        }
        _ => (),
      }
    } else {
      let parameter_integer = value;
      let integer_value = ((parameter_integer >> 1) as i32) ^ -((parameter_integer & 1) as i32);
      if parameter_count % DIMENSION == 0 {
        cursor[0] = match cursor[0].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
      } else {
        cursor[1] = match cursor[1].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
        coordinates.push(Coord {
          x: cursor[0] as f32,
          y: cursor[1] as f32,
        });
      }
      parameter_count -= 1;
    }
  }

  match geom_type {
    GeomType::Linestring => {
      // the last linestring is in coordinates vec
      if !linestrings.is_empty() {
        linestrings.push(LineString::new(coordinates));
        return Ok(MultiLineString::new(linestrings).into());
      }
      Ok(LineString::new(coordinates).into())
    }
    GeomType::Point => Ok(
      MultiPoint(
        coordinates
          .iter()
          .map(|coord| Point::new(coord.x, coord.y))
          .collect(),
      )
      .into(),
    ),
    GeomType::Polygon => {
      if !linestrings.is_empty() {
        // TODO: is this if check really needed? can this be simplified as for linestrings?
        // finish last geometry
        polygons.push(Polygon::new(
          linestrings[0].clone(),
          linestrings[1..].into(),
        ));
      }
      if polygons.len() > 1 {
        return Ok(MultiPolygon::new(polygons).into());
      }
      Ok(polygons.get(0).unwrap().to_owned().into())
    }
    GeomType::Unknown => Err(error::ParserError::new(error::GeomtryError::new())),
  }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {

  use wasm_bindgen::prelude::*;

  impl From<super::feature::Feature> for wasm_bindgen::JsValue {
    fn from(_feature: super::feature::Feature) -> Self {
      JsValue::NULL // TODO: convert to GeoJSON structure
    }
  }

  #[wasm_bindgen]
  pub struct Reader {
    reader: Option<super::Reader>,
  }

  #[wasm_bindgen]
  impl Reader {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Reader {
      let reader = match super::Reader::new(data) {
        Ok(reader) => Some(reader),
        Err(error) => {
          // TODO: Handle error to js side
          println!("{:?}", error);
          None
        }
      };
      Reader { reader }
    }

    #[wasm_bindgen(js_name = getLayerNames)]
    pub fn get_layer_names(&self) -> JsValue {
      match &self.reader {
        Some(reader) => {
          match reader.get_layer_names() {
            Ok(layer_names) => JsValue::from(
              layer_names
                .into_iter()
                .map(JsValue::from)
                .collect::<js_sys::Array>(),
            ),
            Err(error) => {
              // TODO: Handle error to js side
              println!("{:?}", error);
              JsValue::NULL
            }
          }
        }
        None => JsValue::NULL,
      }
    }

    #[wasm_bindgen(js_name = getFeatures)]
    pub fn get_features(&self, layer_index: usize) -> JsValue {
      match &self.reader {
        Some(reader) => {
          match reader.get_features(layer_index) {
            Ok(features) => JsValue::from(
              features
                .into_iter()
                .map(JsValue::from)
                .collect::<js_sys::Array>(),
            ),
            Err(error) => {
              // TODO: Handle error to js side
              println!("{:?}", error);
              JsValue::NULL
            }
          }
        }
        None => JsValue::NULL,
      }
    }
  }
}