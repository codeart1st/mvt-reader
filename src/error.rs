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
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParserError {
  /// A protobuf decoding failure.
  #[error("Decode error: {0}")]
  Decode(#[from] prost::DecodeError),

  /// The layer has an unsupported vector tile version.
  #[error("Vector tile version not supported for layer `{layer_name}` (found version: {version})")]
  UnsupportedVersion {
    layer_name: String,
    version: u32,
  },

  /// The tags section of a feature is malformed.
  #[error("Tags section contains errors")]
  InvalidTags,

  /// The geometry section of a feature is malformed.
  #[error("Geometry section contains errors")]
  InvalidGeometry,
}
