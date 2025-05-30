//! This module provides types and utilities for working with features in the `mvt-reader` crate.
//!
//! A feature represents a geographic entity with geometry and associated properties. Features are typically found within layers of a vector tile.
//!
//! # Types
//!
//! The `feature` module defines the following types:
//!
//! - `Feature`: Represents a feature with geometry and properties.

use crate::{
  error::{self, ParserError},
  geometry::{CoordinateStorage, CoordinateTransform, GeometryIterator},
  parse_tags,
  tile::{self, GeomType},
};
use geo_types::Geometry as GeoTypesGeometry;

/// A structure representing a feature in a vector tile.
pub struct Feature<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  /// The geometry of the feature.
  pub geometry: GeometryIterator<'a, S, T>,

  /// Optional properties associated with the feature.
  pub properties: Option<serde_json::Map<String, serde_json::Value>>,
}

impl<'a, S, T> Feature<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  /// Construct a feature from actual MVT feature.
  pub fn from_raw(
    layer: &tile::Layer,
    raw: &'a tile::Feature,
    transform: T,
  ) -> Result<Self, error::ParserError> {
    if let Some(geom_type) = raw.r#type {
      match GeomType::try_from(geom_type) {
        Ok(geom_type) => {
          let parsed_geometry = GeometryIterator::new(&raw.geometry, geom_type, transform);

          let parsed_tags = match parse_tags(&raw.tags, &layer.keys, &layer.values) {
            Ok(parsed_tags) => parsed_tags,
            Err(error) => {
              return Err(error);
            }
          };

          return Ok(Feature {
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
    Err(ParserError::new(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "Parse error",
    )))
  }
}

/// A structure representing a feature in a vector tile.
pub struct LegacyFeature {
  /// The geometry of the feature.
  pub geometry: GeoTypesGeometry<f32>,

  /// Optional properties associated with the feature.
  pub properties: Option<serde_json::Map<String, serde_json::Value>>,
}

impl LegacyFeature {
  /// Retrieves the geometry of the feature.
  ///
  /// # Returns
  ///
  /// A reference to the geometry of the feature.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::feature::Feature;
  /// use geo_types::{Geometry, Point};
  ///
  /// let feature = Feature {
  ///   geometry: Geometry::Point(Point::new(0.0, 0.0)),
  ///   properties: None,
  /// };
  ///
  /// let geometry = feature.get_geometry();
  /// println!("{:?}", geometry);
  /// ```
  pub fn get_geometry(&self) -> &GeoTypesGeometry<f32> {
    &self.geometry
  }
}
