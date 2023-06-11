/// A structure representing a parser error.
#[derive(Debug)]
pub struct ParserError {
  source: Box<dyn std::error::Error>,
}

impl ParserError {
  /// Creates a new `ParserError` instance with the provided error source.
  ///
  /// # Arguments
  ///
  /// * `source` - The underlying error source.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::ParserError;
  ///
  /// let source_error = std::io::Error::new(std::io::ErrorKind::Other, "Custom error");
  /// let parser_error = ParserError::new(source_error);
  /// ```
  pub fn new<T: std::error::Error + 'static>(source: T) -> Self {
    Self {
      source: Box::new(source),
    }
  }
}

impl std::fmt::Display for ParserError {
  /// Formats the error message associated with the `ParserError`.
  ///
  /// # Arguments
  ///
  /// * `f` - The formatter to write the output to.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::error::Error;
  /// use mvt_reader::error::ParserError;
  ///
  /// let source_error = std::io::Error::new(std::io::ErrorKind::Other, "Custom error");
  /// let parser_error = ParserError::new(source_error);
  /// println!("{}", parser_error);
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.source.fmt(f)
  }
}

impl std::error::Error for ParserError {
  /// Returns the underlying source of the `ParserError`.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::error::Error;
  /// use mvt_reader::error::ParserError;
  ///
  /// let source_error = std::io::Error::new(std::io::ErrorKind::Other, "Custom error");
  /// let parser_error = ParserError::new(source_error);
  /// let source = parser_error.source();
  /// println!("Source: {}", source.unwrap());
  /// ```
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(self.source.as_ref())
  }
}

/// A structure representing a version error in a vector tile.
#[derive(Debug)]
pub struct VersionError {
  layer_name: String,

  version: u32,
}

impl VersionError {
  /// Creates a new `VersionError` instance with the provided layer name and version.
  ///
  /// # Arguments
  ///
  /// * `layer_name` - The name of the layer.
  /// * `version` - The unsupported version number.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::VersionError;
  ///
  /// let layer_name = String::from("my_layer");
  /// let version = 3;
  /// let version_error = VersionError::new(layer_name, version);
  /// ```
  pub fn new(layer_name: String, version: u32) -> Self {
    Self {
      layer_name,
      version,
    }
  }
}

impl std::fmt::Display for VersionError {
  /// Formats the error message associated with the `VersionError`.
  ///
  /// # Arguments
  ///
  /// * `f` - The formatter to write the output to.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::VersionError;
  ///
  /// let layer_name = String::from("my_layer");
  /// let version = 3;
  /// let version_error = VersionError::new(layer_name, version);
  /// println!("{}", version_error);
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Vector tile version not supported for layer `{}` (found version: {})",
      self.layer_name, self.version
    )
  }
}

impl std::error::Error for VersionError {}

/// An error indicating that the tags section of a vector tile contains errors.
#[derive(Debug, Default)]
pub struct TagsError;

impl TagsError {
  /// Creates a new `TagsError` instance.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::TagsError;
  ///
  /// let tags_error = TagsError::new();
  /// ```
  pub fn new() -> Self {
    Self
  }
}

impl std::fmt::Display for TagsError {
  /// Formats the error message associated with the `TagsError`.
  ///
  /// # Arguments
  ///
  /// * `f` - The formatter to write the output to.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::TagsError;
  ///
  /// let tags_error = TagsError::new();
  /// println!("{}", tags_error);
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Tags section contains errors")
  }
}

impl std::error::Error for TagsError {}

/// An error indicating that the geometry section of a vector tile contains errors.
#[derive(Debug, Default)]
pub struct GeometryError;

impl GeometryError {
  /// Creates a new `GeometryError` instance.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::GeometryError;
  ///
  /// let geometry_error = GeometryError::new();
  /// ```
  pub fn new() -> Self {
    Self
  }
}

impl std::fmt::Display for GeometryError {
  /// Formats the error message associated with the `GeometryError`.
  ///
  /// # Arguments
  ///
  /// * `f` - The formatter to write the output to.
  ///
  /// # Examples
  ///
  /// ```
  /// use mvt_reader::error::GeometryError;
  ///
  /// let geometry_error = GeometryError::new();
  /// println!("{}", geometry_error);
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Geometry section contains errors")
  }
}

impl std::error::Error for GeometryError {}
