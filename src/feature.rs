use std::collections::HashMap;

pub struct Feature<T> {
  pub geometry: T,

  pub properties: Option<HashMap<String, String>>,
}

impl<T> Feature<T> {
  pub fn get_geometry(&self) -> &T {
    &self.geometry
  }
}
