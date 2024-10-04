//! This module provides error types and utilities for the `mvt-reader` crate.
//!
//! # Errors
//!
//! The `error` module defines the following error types:
//!
//! - `ParserError`: Represents an error that occurs during parsing of a vector tile.
//! - `GeometryError`: Represents an error related to the geometry of a feature in a vector tile.
//! - `TagsError`: Represents an error related to the tags of a feature in a vector tile.
//! - `VersionError`: Represents an error related to the version of a vector tile.
//! - `DecodeError`: Represents an error indicating a decoding failure during the parsing of a vector tile.
//!
//! # Utilities
//!
//! The `error` module also provides utility functions and traits for working with errors, such as formatting and error chaining.

/// A structure representing a parser error.
#[derive(Debug)]
pub struct ParserError {
  source: Box<dyn core::error::Error>,
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
  pub fn new<T: core::error::Error + 'static>(source: T) -> Self {
    Self {
      source: Box::new(source),
    }
  }
}

impl core::fmt::Display for ParserError {
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
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.source.fmt(f)
  }
}

impl core::error::Error for ParserError {
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
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
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

impl core::fmt::Display for VersionError {
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
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "Vector tile version not supported for layer `{}` (found version: {})",
      self.layer_name, self.version
    )
  }
}

impl core::error::Error for VersionError {}

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

impl core::fmt::Display for TagsError {
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
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Tags section contains errors")
  }
}

impl core::error::Error for TagsError {}

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

impl core::fmt::Display for GeometryError {
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
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Geometry section contains errors")
  }
}

impl core::error::Error for GeometryError {}

/// An error indicating a decoding failure during the parsing of a vector tile.
#[derive(Debug)]
pub struct DecodeError {
  source: Box<dyn core::error::Error>,
}

impl DecodeError {
  /// Creates a new DecodeError instance with the provided decoding error from prost.
  ///
  /// # Arguments
  ///
  /// * source - The underlying decoding error from prost.
  pub fn new(source: Box<dyn core::error::Error>) -> Self {
    Self { source }
  }
}

impl core::fmt::Display for DecodeError {
  /// Formats the error message associated with the `DecodeError`.
  ///
  /// # Arguments
  ///
  /// * `f` - The formatter to write the output to.
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Decode error: {}", self.source)
  }
}

impl core::error::Error for DecodeError {}
