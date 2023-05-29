use std::collections::HashMap;

use geo_types::Geometry;

pub struct Feature {
  pub geometry: Geometry<f32>,

  pub properties: Option<HashMap<String, String>>,
}

impl Feature {
  pub fn get_geometry(&self) -> &Geometry<f32> {
    &self.geometry
  }
}
