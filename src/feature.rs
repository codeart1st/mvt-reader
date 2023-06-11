use std::collections::HashMap;

use geo_types::Geometry;

/// A structure representing a feature in a vector tile.
pub struct Feature {
  /// The geometry of the feature.
  pub geometry: Geometry<f32>,

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
  /// let feature = Feature {
  ///     geometry: Geometry::Point(Point::new(0.0, 0.0)),
  ///     properties: None,
  /// };
  ///
  /// let geometry = feature.get_geometry();
  /// println!("{:?}", geometry);
  /// ```
  pub fn get_geometry(&self) -> &Geometry<f32> {
    &self.geometry
  }
}
