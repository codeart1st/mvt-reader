#[derive(Debug)]
pub struct ParserError {
  source: Box<dyn std::error::Error>,
}

impl ParserError {
  pub fn new<T: std::error::Error + 'static>(source: T) -> Self {
    Self {
      source: Box::new(source),
    }
  }
}

impl std::fmt::Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.source.fmt(f)
  }
}

impl std::error::Error for ParserError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(self.source.as_ref())
  }
}

#[derive(Debug)]
pub struct VersionError {
  layer_name: String,

  version: u32,
}

impl VersionError {
  pub fn new(layer_name: String, version: u32) -> Self {
    Self {
      layer_name,
      version,
    }
  }
}

impl std::fmt::Display for VersionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Vector tile version not supported for layer `{}` (found version: {})",
      self.layer_name, self.version
    )
  }
}

impl std::error::Error for VersionError {}

#[derive(Debug, Default)]
pub struct TagsError;

impl TagsError {
  pub fn new() -> Self {
    Self
  }
}

impl std::fmt::Display for TagsError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Tags section contains errors")
  }
}

impl std::error::Error for TagsError {}

#[derive(Debug, Default)]
pub struct GeomtryError;

impl GeomtryError {
  pub fn new() -> Self {
    Self
  }
}

impl std::fmt::Display for GeomtryError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Geometry section contains errors")
  }
}

impl std::error::Error for GeomtryError {}
