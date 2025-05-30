use std::iter::Iterator;

use geo_types::{
  Coord, Geometry as GeoTypesGeometry, LineString, MultiLineString, MultiPoint, MultiPolygon,
  Point, Polygon,
};

use crate::{error, tile::GeomType};

/// The dimension used for the vector tile.
const DIMENSION: u32 = 2;

/// Trait representing the type of transformation output
pub trait TransformOutput: Clone + Copy {
  /// Add coordinates to a Vec<f32>
  fn push_to_vec(&self, vec: &mut Vec<f32>);

  /// Get the number of dimensions
  fn dimensions(&self) -> usize;
}

/// Output for 2D coordinates
impl TransformOutput for (f32, f32) {
  fn push_to_vec(&self, vec: &mut Vec<f32>) {
    vec.push(self.0);
    vec.push(self.1);
  }

  fn dimensions(&self) -> usize {
    2
  }
}

/// Output for 3D coordinates
impl TransformOutput for (f32, f32, f32) {
  fn push_to_vec(&self, vec: &mut Vec<f32>) {
    vec.push(self.0);
    vec.push(self.1);
    vec.push(self.2);
  }

  fn dimensions(&self) -> usize {
    3
  }
}

/// Trait for performing coordinate transformations
pub trait CoordinateTransform {
  /// The type of transformed coordinates (e.g., (f32, f32) for 2D, (f32, f32, f32) for 3D)
  type Output: TransformOutput;

  /// Transform coordinates
  fn transform(&self, x: f32, y: f32, geom_type: &GeomType) -> Self::Output;
}

/// Trait for storing coordinates
pub trait CoordinateStorage: Sized {
  /// The type of transformed coordinates
  type TransformedCoord: TransformOutput;

  /// Add coordinates (must be called by the implementation)
  fn push_coord(&mut self, x: f32, y: f32, transformed: Self::TransformedCoord);

  /// Get the first coordinate
  fn first(&self) -> Option<(f32, f32)>;

  /// Get the last coordinate
  fn last(&self) -> Option<(f32, f32)>;

  /// Clear all coordinates (must be called by the implementation)
  fn clear_coords(&mut self);

  /// Get the number of coordinates
  fn len(&self) -> usize;

  /// Check if empty
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Create a new empty instance
  fn new_empty() -> Self;

  /// Get the accumulated area
  fn accumulated_area(&self) -> f32;

  /// Set the accumulated area
  fn set_accumulated_area(&mut self, area: f32);

  /// Add coordinates and accumulate area (default implementation)
  fn push(&mut self, x: f32, y: f32, transformed: Self::TransformedCoord) {
    // If there's a previous coordinate, accumulate the Shoelace formula term
    if let Some((prev_x, prev_y)) = self.last() {
      let current_area = self.accumulated_area();
      self.set_accumulated_area(current_area + prev_x * y - x * prev_y);
    }
    self.push_coord(x, y, transformed);
  }

  /// Clear all coordinates and reset accumulated area (default implementation)
  fn clear(&mut self) {
    self.clear_coords();
    self.set_accumulated_area(0.0);
  }

  /// Get the accumulated area (including the term for connecting the last and first points in ClosePath)
  fn get_accumulated_area(&self) -> f32 {
    let len = self.len();
    if len >= 2 {
      if let (Some((first_x, first_y)), Some((last_x, last_y))) = (self.first(), self.last()) {
        (self.accumulated_area() + last_x * first_y - first_x * last_y) / 2.0
      } else {
        0.0
      }
    } else {
      0.0
    }
  }

  /// Get transformed coordinates as Vec<f32>
  fn into_transformed_vec(self) -> Vec<f32>;

  /// Get a reference to the transformed coordinates
  fn transformed_as_slice(&self) -> &[f32];
}

/// Identity transform (no transformation)
#[derive(Debug, Copy, Clone)]
pub struct IdentityTransform;

impl CoordinateTransform for IdentityTransform {
  type Output = (f32, f32);

  #[inline]
  fn transform(&self, x: f32, y: f32, _geom_type: &GeomType) -> Self::Output {
    (x, y)
  }
}

/// 3D identity transform
#[derive(Debug, Copy, Clone)]
pub struct IdentityTransform3D;

impl CoordinateTransform for IdentityTransform3D {
  type Output = (f32, f32, f32);

  #[inline]
  fn transform(&self, x: f32, y: f32, _geom_type: &GeomType) -> Self::Output {
    (x, y, 0.0)
  }
}

/// Flat Vec<f32> coordinate storage implementation (2D)
#[derive(Debug, Clone)]
pub struct FlatCoordinateStorage {
  transformed_coords: Vec<f32>,
  coords: Vec<f32>,
  accumulated_area: f32,
}

impl FlatCoordinateStorage {
  pub fn new() -> Self {
    Self {
      transformed_coords: Vec::new(),
      coords: Vec::new(),
      accumulated_area: 0.0,
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      transformed_coords: Vec::with_capacity(capacity * 2),
      coords: Vec::with_capacity(capacity * 2),
      accumulated_area: 0.0,
    }
  }

  /// Extract the internal Vec (transfers ownership)
  pub fn into_vec(self) -> Vec<f32> {
    self.coords
  }

  /// Get a reference to the internal Vec
  pub fn as_slice(&self) -> &[f32] {
    &self.coords
  }
}

impl Default for FlatCoordinateStorage {
  fn default() -> Self {
    Self::new()
  }
}

impl CoordinateStorage for FlatCoordinateStorage {
  type TransformedCoord = (f32, f32);

  fn push_coord(&mut self, x: f32, y: f32, transformed: Self::TransformedCoord) {
    self.coords.push(x);
    self.coords.push(y);
    transformed.push_to_vec(&mut self.transformed_coords);
  }

  fn first(&self) -> Option<(f32, f32)> {
    let len = self.coords.len();
    if len >= 2 {
      Some((self.coords[0], self.coords[1]))
    } else {
      None
    }
  }

  fn last(&self) -> Option<(f32, f32)> {
    let len = self.coords.len();
    if len >= 2 {
      Some((self.coords[len - 2], self.coords[len - 1]))
    } else {
      None
    }
  }

  fn clear_coords(&mut self) {
    self.coords.clear();
    self.transformed_coords.clear();
  }

  fn len(&self) -> usize {
    self.coords.len() / 2
  }

  fn new_empty() -> Self {
    Self::new()
  }

  fn accumulated_area(&self) -> f32 {
    self.accumulated_area
  }

  fn set_accumulated_area(&mut self, area: f32) {
    self.accumulated_area = area;
  }

  fn into_transformed_vec(self) -> Vec<f32> {
    self.transformed_coords
  }

  fn transformed_as_slice(&self) -> &[f32] {
    &self.transformed_coords
  }
}

/// Flat Vec<f32> coordinate storage implementation (3D)
#[derive(Debug, Clone)]
pub struct FlatCoordinateStorage3D {
  transformed_coords: Vec<f32>,
  coords: Vec<f32>,
  accumulated_area: f32,
}

impl FlatCoordinateStorage3D {
  pub fn new() -> Self {
    Self {
      transformed_coords: Vec::new(),
      coords: Vec::new(),
      accumulated_area: 0.0,
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      transformed_coords: Vec::with_capacity(capacity * 3), // 3D
      coords: Vec::with_capacity(capacity * 2),
      accumulated_area: 0.0,
    }
  }

  /// Extract the internal Vec (transfers ownership)
  pub fn into_vec(self) -> Vec<f32> {
    self.coords
  }

  /// Get a reference to the internal Vec
  pub fn as_slice(&self) -> &[f32] {
    &self.coords
  }
}

impl Default for FlatCoordinateStorage3D {
  fn default() -> Self {
    Self::new()
  }
}

impl CoordinateStorage for FlatCoordinateStorage3D {
  type TransformedCoord = (f32, f32, f32);

  fn push_coord(&mut self, x: f32, y: f32, transformed: Self::TransformedCoord) {
    self.coords.push(x);
    self.coords.push(y);
    transformed.push_to_vec(&mut self.transformed_coords);
  }

  fn first(&self) -> Option<(f32, f32)> {
    let len = self.coords.len();
    if len >= 2 {
      Some((self.coords[0], self.coords[1]))
    } else {
      None
    }
  }

  fn last(&self) -> Option<(f32, f32)> {
    let len = self.coords.len();
    if len >= 2 {
      Some((self.coords[len - 2], self.coords[len - 1]))
    } else {
      None
    }
  }

  fn clear_coords(&mut self) {
    self.coords.clear();
    self.transformed_coords.clear();
  }

  fn len(&self) -> usize {
    self.coords.len() / 2
  }

  fn new_empty() -> Self {
    Self::new()
  }

  fn accumulated_area(&self) -> f32 {
    self.accumulated_area
  }

  fn set_accumulated_area(&mut self, area: f32) {
    self.accumulated_area = area;
  }

  fn into_transformed_vec(self) -> Vec<f32> {
    self.transformed_coords
  }

  fn transformed_as_slice(&self) -> &[f32] {
    &self.transformed_coords
  }
}

/// Lightweight geometry type
#[derive(Debug)]
pub enum Geometry<S: CoordinateStorage> {
  Point { x: f32, y: f32 },
  LineString(S),
  Polygon { exterior: S, holes: Vec<S> },
  MultiPoint(S),
  MultiLineString(Vec<S>),
  MultiPolygon(Vec<(S, Vec<S>)>),
}

/// Geometry parser iterator
pub struct GeometryIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  geometry_data: &'a [u32],
  geom_type: GeomType,
  position: usize,
  cursor: [i32; 2],
  current_coordinates: S,
  pending_rings: Vec<S>,
  state: ParserState,
  transform: T,
}

#[derive(Debug, Copy, Clone)]
enum ParserState {
  Initial,
  ReadingCommand,
  ReadingParameters { count: u32, command_id: u8 },
  Finished,
}

impl<'a, S, T> GeometryIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  pub fn new(geometry_data: &'a [u32], geom_type: GeomType, transform: T) -> Self {
    Self {
      geometry_data,
      geom_type,
      position: 0,
      cursor: [0, 0],
      current_coordinates: S::new_empty(),
      pending_rings: Vec::new(),
      state: ParserState::Initial,
      transform,
    }
  }

  /// Parse and return the next geometry
  fn parse_next(&mut self) -> Option<Result<Geometry<S>, error::ParserError>> {
    if self.geom_type == GeomType::Unknown {
      return Some(Err(error::ParserError::new(error::GeometryError::new())));
    }

    loop {
      match self.state {
        ParserState::Initial => {
          self.state = ParserState::ReadingCommand;
        }

        ParserState::ReadingCommand => {
          if self.position >= self.geometry_data.len() {
            return self.finish_parsing();
          }

          let command_integer = self.geometry_data[self.position];
          self.position += 1;

          let command_id = (command_integer & 0x7) as u8;
          let count = (command_integer >> 3) * DIMENSION;

          match command_id {
            1 => {
              // MoveTo
              if self.geom_type == GeomType::Linestring && !self.current_coordinates.is_empty() {
                // Move current coordinates to create a LineString
                let mut linestring = S::new_empty();
                std::mem::swap(&mut linestring, &mut self.current_coordinates);

                self.state = ParserState::ReadingParameters { count, command_id };
                return Some(Ok(Geometry::LineString(linestring)));
              }
              self.state = ParserState::ReadingParameters { count, command_id };
            }

            2 => {
              // LineTo
              self.state = ParserState::ReadingParameters { count, command_id };
            }

            7 => {
              // ClosePath
              if self.current_coordinates.first().is_none() {
                return Some(Err(error::ParserError::new(error::GeometryError::new())));
              }

              // The connection from the last point to the first point is not included in the accumulated area,
              // so get_accumulated_area() will calculate it internally

              // Add ring
              let mut ring = S::new_empty();
              std::mem::swap(&mut ring, &mut self.current_coordinates);

              if self.pending_rings.is_empty() {
                // The first ring is always the exterior ring
                self.pending_rings.push(ring);
              } else {
                // Second and subsequent rings: determine by area
                let area = ring.get_accumulated_area();

                if area > 0.0 {
                  // Move ownership from pending_rings
                  let mut rings = Vec::new();
                  std::mem::swap(&mut rings, &mut self.pending_rings);

                  let mut iter = rings.into_iter();
                  let exterior = iter.next().unwrap();
                  let holes = iter.collect();

                  // Save the current ring as the start of a new polygon
                  self.pending_rings.push(ring);

                  self.state = ParserState::ReadingCommand;

                  if self.geom_type == GeomType::Polygon {
                    return Some(Ok(Geometry::Polygon { exterior, holes }));
                  }
                } else {
                  // Negative area: hole in the current polygon
                  self.pending_rings.push(ring);
                }
              }

              self.state = ParserState::ReadingCommand;
            }

            _ => {
              self.state = ParserState::ReadingCommand;
            }
          }
        }

        ParserState::ReadingParameters {
          mut count,
          command_id,
        } => {
          while count > 0 && self.position < self.geometry_data.len() {
            let parameter_integer = self.geometry_data[self.position];
            self.position += 1;

            let integer_value =
              ((parameter_integer >> 1) as i32) ^ -((parameter_integer & 1) as i32);

            if count % DIMENSION == 0 {
              self.cursor[0] = self.cursor[0].saturating_add(integer_value);
            } else {
              self.cursor[1] = self.cursor[1].saturating_add(integer_value);

              let x = self.cursor[0] as f32;
              let y = self.cursor[1] as f32;

              // Apply coordinate transformation
              let transformed_output = self.transform.transform(x, y, &self.geom_type);
              let transformed_coord = S::TransformedCoord::from(transformed_output);

              // For Point type, return each coordinate individually
              if self.geom_type == GeomType::Point && command_id == 1 {
                count -= 1;

                if count > 0 {
                  self.state = ParserState::ReadingParameters { count, command_id };
                } else {
                  self.state = ParserState::ReadingCommand;
                }

                // To support both 2D/3D, get the first 2 elements from the transformation result
                let mut temp_storage = S::new_empty();
                temp_storage.push(x, y, transformed_coord);
                let transformed_slice = temp_storage.transformed_as_slice();

                return Some(Ok(Geometry::Point {
                  x: transformed_slice[0],
                  y: transformed_slice[1],
                }));
              } else {
                self.current_coordinates.push(x, y, transformed_coord);
              }
            }
            count -= 1;
          }

          self.state = ParserState::ReadingCommand;
        }

        ParserState::Finished => {
          return None;
        }
      }
    }
  }

  /// Processing at the end of parsing
  fn finish_parsing(&mut self) -> Option<Result<Geometry<S>, error::ParserError>> {
    self.state = ParserState::Finished;

    match self.geom_type {
      GeomType::Linestring => {
        if !self.current_coordinates.is_empty() {
          let mut linestring = S::new_empty();
          std::mem::swap(&mut linestring, &mut self.current_coordinates);
          Some(Ok(Geometry::LineString(linestring)))
        } else {
          None
        }
      }

      GeomType::Point => None,

      GeomType::Polygon => {
        if !self.pending_rings.is_empty() {
          let mut exterior = S::new_empty();
          let mut holes = Vec::new();

          let rings = std::mem::take(&mut self.pending_rings);

          if !rings.is_empty() {
            let mut iter = rings.into_iter();
            exterior = iter.next().unwrap();
            holes = iter.collect();
          }

          Some(Ok(Geometry::Polygon { exterior, holes }))
        } else {
          None
        }
      }

      GeomType::Unknown => None,
    }
  }
}

impl<'a, S, T> Iterator for GeometryIterator<'a, S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  type Item = Result<Geometry<S>, error::ParserError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.parse_next()
  }
}

/// Create a geometry iterator
pub fn parse_geometry_iter<S, T>(
  geometry_data: &[u32],
  geom_type: GeomType,
  transform: T,
) -> GeometryIterator<S, T>
where
  S: CoordinateStorage,
  T: CoordinateTransform,
  S::TransformedCoord: From<T::Output>,
{
  GeometryIterator::new(geometry_data, geom_type, transform)
}

fn shoelace_formula(points: &[Point<f32>]) -> f32 {
  let mut area: f32 = 0.0;
  let n = points.len();
  let mut v1 = points[n - 1];
  for v2 in points.iter().take(n) {
    area += (v2.y() - v1.y()) * (v2.x() + v1.x());
    v1 = *v2;
  }
  area * 0.5
}

pub fn parse_geometry(
  geometry_data: &[u32],
  geom_type: GeomType,
) -> Result<GeoTypesGeometry<f32>, error::ParserError> {
  if geom_type == GeomType::Unknown {
    return Err(error::ParserError::new(error::GeometryError::new()));
  }

  // worst case capacity to prevent reallocation. not needed to be exact.
  let mut coordinates: Vec<Coord<f32>> = Vec::with_capacity(geometry_data.len());
  let mut polygons: Vec<Polygon<f32>> = Vec::new();
  let mut linestrings: Vec<LineString<f32>> = Vec::new();

  let mut cursor: [i32; 2] = [0, 0];
  let mut parameter_count: u32 = 0;

  for value in geometry_data.iter() {
    if parameter_count == 0 {
      let command_integer = value;
      let id = (command_integer & 0x7) as u8;
      match id {
        1 => {
          // MoveTo
          parameter_count = (command_integer >> 3) * DIMENSION;
          if geom_type == GeomType::Linestring && !coordinates.is_empty() {
            linestrings.push(LineString::new(coordinates));
            // start with a new linestring
            coordinates = Vec::with_capacity(geometry_data.len());
          }
        }
        2 => {
          // LineTo
          parameter_count = (command_integer >> 3) * DIMENSION;
        }
        7 => {
          // ClosePath
          let first_coordinate = match coordinates.first() {
            Some(coord) => coord.to_owned(),
            None => {
              return Err(error::ParserError::new(error::GeometryError::new()));
            }
          };
          coordinates.push(first_coordinate);

          let ring = LineString::new(coordinates);

          let area = shoelace_formula(&ring.clone().into_points());

          if area > 0.0 {
            // exterior ring
            if !linestrings.is_empty() {
              // finish previous geometry
              polygons.push(Polygon::new(
                linestrings[0].clone(),
                linestrings[1..].into(),
              ));
              linestrings = Vec::new();
            }
          }

          linestrings.push(ring);
          // start a new sequence
          coordinates = Vec::with_capacity(geometry_data.len());
        }
        _ => (),
      }
    } else {
      let parameter_integer = value;
      let integer_value = ((parameter_integer >> 1) as i32) ^ -((parameter_integer & 1) as i32);
      if parameter_count % DIMENSION == 0 {
        cursor[0] = match cursor[0].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
      } else {
        cursor[1] = match cursor[1].checked_add(integer_value) {
          Some(result) => result,
          None => std::i32::MAX, // clip value
        };
        coordinates.push(Coord {
          x: cursor[0] as f32,
          y: cursor[1] as f32,
        });
      }
      parameter_count -= 1;
    }
  }

  match geom_type {
    GeomType::Linestring => {
      // the last linestring is in coordinates vec
      if !linestrings.is_empty() {
        linestrings.push(LineString::new(coordinates));
        return Ok(MultiLineString::new(linestrings).into());
      }
      Ok(LineString::new(coordinates).into())
    }
    GeomType::Point => Ok(
      MultiPoint(
        coordinates
          .iter()
          .map(|coord| Point::new(coord.x, coord.y))
          .collect(),
      )
      .into(),
    ),
    GeomType::Polygon => {
      if !linestrings.is_empty() {
        // finish pending polygon
        polygons.push(Polygon::new(
          linestrings[0].clone(),
          linestrings[1..].into(),
        ));
        return Ok(MultiPolygon::new(polygons).into());
      }
      Ok(polygons.first().unwrap().to_owned().into())
    }
    GeomType::Unknown => Err(error::ParserError::new(error::GeometryError::new())),
  }
}
