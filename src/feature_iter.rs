//! Iterator implementation for features in a vector tile layer.
//!
//! This module provides the `FeatureIterator` struct which allows iterating
//! over features in a layer with custom coordinate storage and transformation.

use crate::{feature::Feature, tile, geometry::{CoordinateStorage, CoordinateTransform}};
use std::marker::PhantomData;

/// An iterator over features in a vector tile layer.
///
/// This iterator applies a coordinate transformation to each feature as it is yielded.
///
/// # Type Parameters
///
/// * `'a` - The lifetime of the layer reference
/// * `S` - The coordinate storage type implementing `CoordinateStorage`
/// * `T` - The coordinate transformation type implementing `CoordinateTransform`
pub struct FeatureIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
{
  layer: &'a tile::Layer,
  idx: usize,
  transform: T,
  _phantom: PhantomData<S>,
}

impl<'a, S, T> FeatureIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
{
  /// Creates a new feature iterator for the given layer.
  ///
  /// # Arguments
  ///
  /// * `layer` - The vector tile layer to iterate over
  /// * `transform` - The coordinate transformation to apply to each feature
  ///
  /// # Returns
  ///
  /// A new `FeatureIterator` instance
  pub fn new(layer: &'a tile::Layer, transform: T) -> Self {
    Self {
      layer,
      idx: 0,
      transform,
      _phantom: PhantomData,
    }
  }
}

impl<'a, S, T> Iterator for FeatureIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform + Clone,
  S::TransformedCoord: From<T::Output>,
{
  type Item = Feature<'a, S, T>;

  /// Advances the iterator and returns the next feature.
  ///
  /// Features that fail to parse are silently skipped.
  ///
  /// # Returns
  ///
  /// - `Some(Feature)` if there is a next feature that parses successfully
  /// - `None` if there are no more features or if parsing fails
  fn next(&mut self) -> Option<Self::Item> {
    let layer = self.layer;
    let feature = self.layer.features.get(self.idx)?;
    self.idx += 1;

    Feature::from_raw(layer, feature, self.transform.clone()).ok()
  }
}
