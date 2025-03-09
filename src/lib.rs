//! # mvt-reader
//!
//! `mvt-reader` is a Rust library for decoding and reading Mapbox vector tiles.
//!
//! It provides the `Reader` struct, which allows you to read vector tiles and access their layers and features.
//!
//! # Usage
//!
//! To use the `mvt-reader` library in your Rust project, add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! mvt-reader = "2.0.1"
//! ```
//!
//! Then, you can import and use the library in your code:
//!
//! ```rust
//! use mvt_reader::{Reader, error::{ParserError}};
//!
//! fn main() -> Result<(), ParserError> {
//!   // Read a vector tile from file or data
//!   let data = vec![/* Vector tile data */];
//!   let reader = Reader::new(data)?;
//!
//!   // Get layer names
//!   let layer_names = reader.get_layer_names()?;
//!   for name in layer_names {
//!     println!("Layer: {}", name);
//!   }
//!
//!   // Get features for a specific layer
//!   let layer_index = 0;
//!   let features = reader.get_features(layer_index)?;
//!   for feature in features {
//!     todo!()
//!   }
//!
//!   Ok(())
//! }
//! ```
//!
//! # Features
//!
//! The `mvt-reader` library provides the following features:
//!
//! - `wasm`: Enables the compilation of the library as a WebAssembly module, allowing usage in JavaScript/TypeScript projects.
//!
//! To enable the `wasm` feature, add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies.mvt-reader]
//! version = "2.0.1"
//! features = ["wasm"]
//! ```
//!
//! # License
//!
//! This project is licensed under the [MIT License](https://github.com/codeart1st/mvt-reader/blob/main/LICENSE).

pub mod error;
pub mod feature;

mod vector_tile;

use feature::{Feature, Value};
use geo_types::{
  Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use prost::{Message, bytes::Bytes};
use vector_tile::{Tile, tile::GeomType};

/// The dimension used for the vector tile.
const DIMENSION: u32 = 2;

/// Reader for decoding and accessing vector tile data.
pub struct Reader {
  tile: Tile,
}

impl Reader {
  /// Creates a new `Reader` instance with the provided vector tile data.
  ///
  /// # Arguments
  ///
  /// * `data` - The vector tile data as a byte vector.
  ///
  /// # Returns
  ///
  /// A result containing the `Reader` instance if successful, or a `DecodeError` if decoding the vector tile data fails.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::Reader;
  ///
  /// let data = vec![/* Vector tile data */];
  /// let reader = Reader::new(data);
  /// ```
  pub fn new(data: Vec<u8>) -> Result<Self, error::ParserError> {
    match Tile::decode(Bytes::from(data)) {
      Ok(tile) => Ok(Self { tile }),
      Err(error) => Err(error::ParserError::new(error::DecodeError::new(Box::new(
        error,
      )))),
    }
  }

  /// Retrieves the names of the layers in the vector tile.
  ///
  /// # Returns
  ///
  /// A result containing a vector of layer names if successful, or a `ParserError` if there is an error parsing the tile.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::Reader;
  ///
  /// let data = vec![/* Vector tile data */];
  /// let reader = Reader::new(data).unwrap();
  ///
  /// match reader.get_layer_names() {
  ///   Ok(layer_names) => {
  ///     for name in layer_names {
  ///       println!("Layer: {}", name);
  ///     }
  ///   }
  ///   Err(error) => {
  ///     todo!();
  ///   }
  /// }
  /// ```
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
          )));
        }
      }
    }
    Ok(layer_names)
  }

  /// Retrieves the features of a specific layer in the vector tile.
  ///
  /// # Arguments
  ///
  /// * `layer_index` - The index of the layer.
  ///
  /// # Returns
  ///
  /// A result containing a vector of features if successful, or a `ParserError` if there is an error parsing the tile or accessing the layer.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::Reader;
  ///
  /// let data = vec![/* Vector tile data */];
  /// let reader = Reader::new(data).unwrap();
  ///
  /// match reader.get_features(0) {
  ///   Ok(features) => {
  ///     for feature in features {
  ///       todo!();
  ///     }
  ///   }
  ///   Err(error) => {
  ///     todo!();
  ///   }
  /// }
  /// ```
  pub fn get_features(&self, layer_index: usize) -> Result<Vec<Feature>, error::ParserError> {
    let layer = self.tile.layers.get(layer_index);
    match layer {
      Some(layer) => {
        let mut features = Vec::with_capacity(layer.features.len());
        for feature in layer.features.iter() {
          if let Some(geom_type) = feature.r#type {
            match GeomType::try_from(geom_type) {
              Ok(geom_type) => {
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
                  id: feature.id,
                  properties: Some(parsed_tags),
                });
              }
              Err(error) => {
                return Err(error::ParserError::new(error::DecodeError::new(Box::new(
                  error,
                ))));
              }
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
) -> Result<std::collections::HashMap<String, Value>, error::ParserError> {
  let mut result = std::collections::HashMap::new();
  for item in tags.chunks(2) {
    if item.len() != 2
      || item[0] >= keys.len().try_into().unwrap()
      || item[1] >= values.len().try_into().unwrap()
    {
      return Err(error::ParserError::new(error::TagsError::new()));
    }
    result.insert(
      keys[item[0] as usize].clone(),
      map_value(values[item[1] as usize].clone()),
    );
  }
  Ok(result)
}

fn map_value(value: vector_tile::tile::Value) -> Value {
  if let Some(s) = value.string_value {
    return Value::String(s);
  }
  if let Some(f) = value.float_value {
    return Value::Float(f);
  }
  if let Some(d) = value.double_value {
    return Value::Double(d);
  }
  if let Some(i) = value.int_value {
    return Value::Int(i);
  }
  if let Some(u) = value.uint_value {
    return Value::UInt(u);
  }
  if let Some(s) = value.sint_value {
    return Value::SInt(s);
  }
  if let Some(b) = value.bool_value {
    return Value::Bool(b);
  }
  Value::Null
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
    return Err(error::ParserError::new(error::GeometryError::new()));
  }

  // worst case capacity to prevent reallocation. not needed to be exact.
  let mut coordinates: Vec<Coord<f32>> = Vec::with_capacity(geometry_data.len());
  let mut polygons: Vec<Polygon<f32>> = Vec::new();
  let mut linestrings: Vec<LineString<f32>> = Vec::new();

  let mut cursor: [i32; 2] = [0, 0];
  let mut parameter_count: u32 = 0;

  for value in geometry_data.iter() {
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
              return Err(error::ParserError::new(error::GeometryError::new()));
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
          None => i32::MAX, // clip value
        };
      } else {
        cursor[1] = match cursor[1].checked_add(integer_value) {
          Some(result) => result,
          None => i32::MAX, // clip value
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
        // finish pending polygon
        polygons.push(Polygon::new(
          linestrings[0].clone(),
          linestrings[1..].into(),
        ));
        return Ok(MultiPolygon::new(polygons).into());
      }
      match polygons.first() {
        Some(polygon) => Ok(polygon.to_owned().into()),
        None => Err(error::ParserError::new(error::GeometryError::new())),
      }
    }
    GeomType::Unknown => Err(error::ParserError::new(error::GeometryError::new())),
  }
}

#[cfg(feature = "wasm")]
pub mod wasm {

  use crate::feature::Value;
  use geojson::{Feature, GeoJson, JsonObject, JsonValue, feature::Id};
  use serde::Serialize;
  use serde_wasm_bindgen::Serializer;
  use wasm_bindgen::prelude::*;

  impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
      match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::from(b),
        Value::Int(i) => JsonValue::from(i),
        Value::UInt(u) => JsonValue::from(u),
        Value::SInt(s) => JsonValue::from(s),
        Value::Float(f) => JsonValue::from(f),
        Value::Double(d) => JsonValue::from(d),
        Value::String(s) => JsonValue::from(s),
      }
    }
  }

  /// Converts a `super::feature::Feature` into a `wasm_bindgen::JsValue`.
  impl From<super::feature::Feature> for wasm_bindgen::JsValue {
    fn from(feature: super::feature::Feature) -> Self {
      let properties: Option<JsonObject> = feature.properties.as_ref().map(|props| {
        props
          .clone()
          .into_iter()
          .map(|(k, v)| (k, v.into()))
          .collect()
      });

      let geojson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(feature.get_geometry().into()),
        id: feature.id.map(|id| Id::Number(id.into())),
        properties,
        foreign_members: None,
      });

      geojson.serialize(&Serializer::json_compatible()).unwrap()
    }
  }

  /// Reader for decoding and accessing vector tile data in WebAssembly.
  #[wasm_bindgen]
  pub struct Reader {
    reader: Option<super::Reader>,
  }

  #[wasm_bindgen]
  impl Reader {
    /// Creates a new `Reader` instance with the provided vector tile data.
    ///
    /// # Arguments
    ///
    /// * `data` - The vector tile data as a `Vec<u8>`.
    /// * `error_callback` - An optional JavaScript callback function to handle errors. It should accept a single parameter which will contain the error message as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let tileData = getVectorTileData();
    /// let reader = new Reader(tileData, handleErrors);
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>, error_callback: Option<js_sys::Function>) -> Reader {
      let reader = match super::Reader::new(data) {
        Ok(reader) => Some(reader),
        Err(error) => {
          if let Some(callback) = error_callback {
            callback
              .call1(&JsValue::NULL, &JsValue::from_str(&format!("{:?}", error)))
              .unwrap();
          }
          None
        }
      };
      Reader { reader }
    }

    /// Retrieves the layer names present in the vector tile.
    ///
    /// # Arguments
    ///
    /// * `error_callback` - An optional JavaScript callback function to handle errors. It should accept a single parameter which will contain the error message as a string.
    ///
    /// # Returns
    ///
    /// A JavaScript array containing the layer names as strings.
    ///
    /// # Examples
    ///
    /// ```
    /// let layerNames = reader.getLayerNames(handleErrors);
    /// for (let i = 0; i < layerNames.length; i++) {
    ///   console.log(layerNames[i]);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getLayerNames)]
    pub fn get_layer_names(&self, error_callback: Option<js_sys::Function>) -> JsValue {
      match &self.reader {
        Some(reader) => match reader.get_layer_names() {
          Ok(layer_names) => JsValue::from(
            layer_names
              .into_iter()
              .map(JsValue::from)
              .collect::<js_sys::Array>(),
          ),
          Err(error) => {
            if let Some(callback) = error_callback {
              callback
                .call1(&JsValue::NULL, &JsValue::from_str(&format!("{:?}", error)))
                .unwrap();
            }
            JsValue::NULL
          }
        },
        None => JsValue::NULL,
      }
    }

    /// Retrieves the features of a specific layer in the vector tile.
    ///
    /// # Arguments
    ///
    /// * `layer_index` - The index of the layer to retrieve features from.
    /// * `error_callback` - An optional JavaScript callback function to handle errors. It should accept a single parameter which will contain the error message as a string.
    ///
    /// # Returns
    ///
    /// A JavaScript array containing the features as GeoJSON objects.
    ///
    /// # Examples
    ///
    /// ```
    /// let features = reader.getFeatures(0, handleErrors);
    /// for (let i = 0; i < features.length; i++) {
    ///   console.log(features[i]);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getFeatures)]
    pub fn get_features(
      &self,
      layer_index: usize,
      error_callback: Option<js_sys::Function>,
    ) -> JsValue {
      match &self.reader {
        Some(reader) => match reader.get_features(layer_index) {
          Ok(features) => JsValue::from(
            features
              .into_iter()
              .map(JsValue::from)
              .collect::<js_sys::Array>(),
          ),
          Err(error) => {
            if let Some(callback) = error_callback {
              callback
                .call1(&JsValue::NULL, &JsValue::from_str(&format!("{:?}", error)))
                .unwrap();
            }
            JsValue::NULL
          }
        },
        None => JsValue::NULL,
      }
    }
  }
}
