//! This module provide the `Layer` struct.
//!
//! The `Layer` struct represents a layer in a vector tile, containing metadata about the layer and its features.
//!
//! # Types
//!
//! The `layer` module defines the following types:
//!
//! - `Layer`: Represents a layer in a vector tile, containing metadata about the layer and its features.

/// A structure representing a layer in a vector tile.
#[derive(Debug, Clone)]
pub struct Layer {
  /// The index of the layer in the vector tile.
  pub layer_index: usize,

  /// The version of the layer.
  pub version: u32,

  /// The name of the layer.
  pub name: String,

  /// The number of features in the layer.
  pub feature_count: usize,

  /// The extent of the layer, representing the size of the tile in pixels. Defaults to 4096.
  pub extent: u32,
}
