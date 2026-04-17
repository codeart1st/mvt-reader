//! This module provides error types for the `mvt-reader` crate.
//!
//! The [`ParserError`] enum represents all possible errors that can occur during
//! parsing of a vector tile.

/// An error that can occur during parsing of a vector tile.
///
/// # Examples
///
/// ```
/// use mvt_reader::error::ParserError;
///
/// fn example() -> Result<(), ParserError> {
///   Err(ParserError::InvalidTags)
/// }
/// ```
#[derive(Debug)]
pub enum ParserError {
  /// A protobuf decoding failure.
  Decode(prost::DecodeError),

  /// The layer has an unsupported vector tile version.
  UnsupportedVersion {
    layer_name: String,
    version: u32,
  },

  /// The tags section of a feature is malformed.
  InvalidTags,

  /// The geometry section of a feature is malformed.
  InvalidGeometry,
}

impl core::fmt::Display for ParserError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Decode(source) => write!(f, "Decode error: {source}"),
      Self::UnsupportedVersion { layer_name, version } => write!(
        f,
        "Vector tile version not supported for layer `{layer_name}` (found version: {version})"
      ),
      Self::InvalidTags => write!(f, "Tags section contains errors"),
      Self::InvalidGeometry => write!(f, "Geometry section contains errors"),
    }
  }
}

impl core::error::Error for ParserError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Decode(source) => Some(source),
      _ => None,
    }
  }
}

impl From<prost::DecodeError> for ParserError {
  fn from(error: prost::DecodeError) -> Self {
    Self::Decode(error)
  }
}
