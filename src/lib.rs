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
//! mvt-reader = "1.5.0"
//! ```
//!
//! Then, you can import and use the library in your code:
//!
//! ```no_run
//! use mvt_reader::{Reader, FlatCoordinateStorage, IdentityTransform, error::{ParserError}};
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
//!   let features = reader.get_features_iter::<FlatCoordinateStorage, _>(layer_index, IdentityTransform).unwrap();
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
//! version = "1.5.0"
//! features = ["wasm"]
//! ```
//!
//! # License
//!
//! This project is licensed under the [MIT License](https://github.com/codeart1st/mvt-reader/blob/main/LICENSE).

pub mod error;
pub mod feature;
pub mod feature_iter;
pub mod geometry;

mod vector_tile;

use feature::LegacyFeature;
use feature_iter::FeatureIterator;
use geometry::{parse_geometry, CoordinateStorage, CoordinateTransform};
pub use prost::{bytes::Bytes, Message};

use vector_tile::tile::GeomType;
pub use vector_tile::*;

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
          )))
        }
      }
    }
    Ok(layer_names)
  }

  /// Get features iterator with custom coordinate storage and transform
  pub fn get_features_iter<S, T>(
    &self,
    layer_index: usize,
    transform: T,
  ) -> Option<FeatureIterator<S, T>>
  where
    S: CoordinateStorage,
    T: CoordinateTransform,
  {
    let layer = self.tile.layers.get(layer_index)?;
    Some(FeatureIterator::new(layer, transform))
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
  pub fn get_features(&self, layer_index: usize) -> Result<Vec<LegacyFeature>, error::ParserError> {
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

                features.push(LegacyFeature {
                  geometry: parsed_geometry,
                  properties: Some(parsed_tags),
                });
              }
              Err(error) => {
                return Err(error::ParserError::new(error::DecodeError::new(Box::new(
                  error,
                ))))
              }
            }
          }
        }
        Ok(features)
      }
      None => Ok(vec![]),
    }
  }

  /// Retrieves the extent of the layers in the vector tile.
  ///
  /// # Returns
  ///
  /// A u32 value. default 4096
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
  ///     for (index, _name) in layer_names.iter().enumerate() {
  ///       let extent = reader.get_extent(index);
  ///       println!("extent: {}", extent);
  ///     }
  ///   }
  ///   Err(error) => {
  ///     todo!();
  ///   }
  /// }
  /// ```
  pub fn get_extent(&self, layer_index: usize) -> u32 {
    self
      .tile
      .layers
      .get(layer_index)
      .and_then(|layer| layer.extent)
      .unwrap_or(4096)
  }
}

fn parse_tags(
  tags: &[u32],
  keys: &[String],
  values: &[tile::Value],
) -> Result<serde_json::Map<String, serde_json::Value>, error::ParserError> {
  let mut result = serde_json::Map::with_capacity(tags.len() / 2);
  for item in tags.chunks(2) {
    if item.len() != 2
      || item[0] > keys.len().try_into().unwrap()
      || item[1] > values.len().try_into().unwrap()
    {
      return Err(error::ParserError::new(error::TagsError::new()));
    }
    result.insert(
      (*keys.get(item[0] as usize).expect("item not found")).clone(),
      serde_json::Value::String(get_string_value(
        (*values.get(item[1] as usize).expect("item not found")).clone(),
      )),
    );
  }
  Ok(result)
}

fn get_string_value(value: tile::Value) -> String {
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

#[cfg(feature = "wasm")]
pub mod wasm {

  use geojson::{Feature, GeoJson, JsonObject};
  use serde::Serialize;
  use serde_wasm_bindgen::Serializer;
  use wasm_bindgen::prelude::*;

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
        id: None,
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
