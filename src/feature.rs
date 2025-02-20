//! This module provides types and utilities for working with features in the `mvt-reader` crate.
//!
//! A feature represents a geographic entity with geometry and associated properties. Features are typically found within layers of a vector tile.
//!
//! # Types
//!
//! The `feature` module defines the following types:
//!
//! - `Feature`: Represents a feature with geometry and properties.

use std::collections::HashMap;

use geo_types::Geometry;

/// A structure representing a feature in a vector tile.
pub struct Feature {
  /// The geometry of the feature.
  pub geometry: Geometry<f32>,

  /// Optional identifier for the feature.
  pub id: Option<u64>,

  /// Optional properties associated with the feature.
  pub properties: Option<HashMap<String, String>>,
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
  ///   id: None,
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
