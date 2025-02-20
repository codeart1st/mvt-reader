//! This module provides types and utilities for working with features in the `mvt-reader` crate.
//!
//! A feature represents a geographic entity with geometry and associated properties. Features are typically found within layers of a vector tile.
//!
//! # Types
//!
//! The `feature` module defines the following types:
//!
//! - `Feature`: Represents a feature with geometry and properties.

use geo_types::Geometry;
use serde_json::{Map, Value};

/// A structure representing a feature in a vector tile.
pub struct Feature {
  pub id: Option<u64>,

  /// The geometry of the feature.
  pub geometry: Geometry<f32>,

  /// Optional properties associated with the feature.
  pub properties: Option<Map<String, Value>>,
}

impl Feature {
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
  pub fn get_geometry(&self) -> &Geometry<f32> {
    &self.geometry
  }
}
