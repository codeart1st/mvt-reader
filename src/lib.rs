mod error;
mod feature;
mod vector_tile;

use feature::Feature;
use geo_types::{Coord, GeometryCollection, LineString, Point, Polygon};
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

  pub fn get_features(
    &self,
    layer_index: usize,
  ) -> Result<Vec<Feature<GeometryCollection<f32>>>, error::ParserError> {
    let layer = self.tile.layers.get(layer_index);
    match layer {
      Some(layer) => {
        let mut features = Vec::with_capacity(layer.features.len());
        for feature in layer.features.iter() {
          if let Some(geom_type) = feature.r#type {
            if let Some(geom_type) = vector_tile::tile::GeomType::from_i32(geom_type) {
              let parsed_geometries = match parse_geometry(&feature.geometry, geom_type) {
                Ok(parsed_geometries) => parsed_geometries,
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
                geometry: parsed_geometries,
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
  _geom_type: vector_tile::tile::GeomType,
) -> Result<GeometryCollection<f32>, error::ParserError> {
  // worst case capacity to prevent reallocation. not needed to be exact.
  let mut coordinates: Vec<Coord<f32>> = Vec::with_capacity(geometry_data.len());
  let mut rings: Vec<LineString<f32>> = Vec::new();
  let mut geometries = Vec::new();

  let mut cursor: [i32; 2] = [0, 0];
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
          let first_coordinate = match coordinates.first() {
            Some(coord) => coord.to_owned(),
            None => {
              return Err(error::ParserError::new(error::GeomtryError::new()));
            }
          };
          coordinates.push(first_coordinate);

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
        cursor[0] = match cursor[0].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
      } else {
        cursor[1] = match cursor[1].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
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
  Ok(GeometryCollection(geometries))
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {

  use wasm_bindgen::prelude::*;

  impl From<super::feature::Feature<geo_types::GeometryCollection<f32>>> for wasm_bindgen::JsValue {
    fn from(_feature: super::feature::Feature<geo_types::GeometryCollection<f32>>) -> Self {
      JsValue::NULL
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
