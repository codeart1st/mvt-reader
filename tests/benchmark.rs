use std::fs::read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use geo_types::Geometry as GeoTypesGeometry;
use mvt_reader::{
  geometry::{CoordinateStorage, CoordinateTransform, FlatCoordinateStorage, Geometry},
  tile::GeomType,
  Reader,
};

/// Scale transformation that performs actual computation
#[derive(Debug, Copy, Clone)]
struct ScaleTransform {
  scale: f32,
}

impl CoordinateTransform for ScaleTransform {
  type Output = (f32, f32);

  #[inline]
  fn transform(&self, x: f32, y: f32, _geom_type: &GeomType) -> Self::Output {
    // Perform some computation to simulate real transformation work
    (x * self.scale + 0.5, y * self.scale + 0.5)
  }
}

#[derive(Debug, Default)]
struct GeometryStats {
  points: usize,
  linestrings: usize,
  polygons: usize,
  exterior_rings: usize,
  interior_rings: usize,
}

// Helper function to apply transformation to legacy geometry
fn apply_transform_to_legacy_geometry(geom: &GeoTypesGeometry<f32>, transform: &ScaleTransform) {
  match geom {
    GeoTypesGeometry::Point(p) => {
      let (x, y) = transform.transform(p.x(), p.y(), &GeomType::Point);
      let _ = (x, y);
    }
    GeoTypesGeometry::MultiPoint(mp) => {
      for p in &mp.0 {
        let (x, y) = transform.transform(p.x(), p.y(), &GeomType::Point);
        let _ = (x, y);
      }
    }
    GeoTypesGeometry::LineString(ls) => {
      for coord in ls.coords() {
        let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Linestring);
        let _ = (x, y);
      }
    }
    GeoTypesGeometry::MultiLineString(mls) => {
      for ls in &mls.0 {
        for coord in ls.coords() {
          let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Linestring);
          let _ = (x, y);
        }
      }
    }
    GeoTypesGeometry::Polygon(p) => {
      for coord in p.exterior().coords() {
        let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Polygon);
        let _ = (x, y);
      }
      for interior in p.interiors() {
        for coord in interior.coords() {
          let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Polygon);
          let _ = (x, y);
        }
      }
    }
    GeoTypesGeometry::MultiPolygon(mp) => {
      for p in &mp.0 {
        for coord in p.exterior().coords() {
          let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Polygon);
          let _ = (x, y);
        }
        for interior in p.interiors() {
          for coord in interior.coords() {
            let (x, y) = transform.transform(coord.x, coord.y, &GeomType::Polygon);
            let _ = (x, y);
          }
        }
      }
    }
    _ => {}
  }
}

impl GeometryStats {
  fn total_geometries(&self) -> usize {
    self.points + self.linestrings + self.polygons
  }

  fn count_legacy_geometry(&mut self, geom: &GeoTypesGeometry<f32>) {
    match geom {
      GeoTypesGeometry::Point(_) => self.points += 1,
      GeoTypesGeometry::MultiPoint(mp) => self.points += mp.0.len(),
      GeoTypesGeometry::LineString(_) => self.linestrings += 1,
      GeoTypesGeometry::MultiLineString(mls) => self.linestrings += mls.0.len(),
      GeoTypesGeometry::Polygon(p) => {
        self.polygons += 1;
        self.exterior_rings += 1;
        self.interior_rings += p.interiors().len();
      }
      GeoTypesGeometry::MultiPolygon(mp) => {
        for p in &mp.0 {
          self.polygons += 1;
          self.exterior_rings += 1;
          self.interior_rings += p.interiors().len();
        }
      }
      _ => {}
    }
  }

  fn count_iter_geometry<S: CoordinateStorage>(&mut self, geom: &Geometry<S>) {
    match geom {
      Geometry::Point { .. } => self.points += 1,
      Geometry::LineString(_) => self.linestrings += 1,
      Geometry::Polygon { holes, .. } => {
        self.polygons += 1;
        self.exterior_rings += 1;
        self.interior_rings += holes.len();
      }
      Geometry::MultiPoint(_) => self.points += 1,
      Geometry::MultiLineString(ls) => self.linestrings += ls.len(),
      Geometry::MultiPolygon(mp) => {
        for (_, holes) in mp {
          self.polygons += 1;
          self.exterior_rings += 1;
          self.interior_rings += holes.len();
        }
      }
    }
  }
}

struct BenchmarkResult {
  method_name: &'static str,
  mean_time: Duration,
  std_dev: Duration,
  min_time: Duration,
  max_time: Duration,
  feature_count: usize,
  geometry_stats: GeometryStats,
  iterations: usize,
}

impl BenchmarkResult {
  fn time_per_feature_us(&self) -> f64 {
    if self.feature_count == 0 {
      0.0
    } else {
      self.mean_time.as_micros() as f64 / self.feature_count as f64
    }
  }

  fn time_per_geometry_us(&self) -> f64 {
    let total_geoms = self.geometry_stats.total_geometries();
    if total_geoms == 0 {
      0.0
    } else {
      self.mean_time.as_micros() as f64 / total_geoms as f64
    }
  }
}

fn calculate_stats(times: &[Duration]) -> (Duration, Duration, Duration, Duration) {
  if times.is_empty() {
    return (Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO);
  }

  let sum: Duration = times.iter().sum();
  let mean = sum / times.len() as u32;
  
  let min = times.iter().min().copied().unwrap_or(Duration::ZERO);
  let max = times.iter().max().copied().unwrap_or(Duration::ZERO);
  
  // Calculate standard deviation
  let variance = times.iter()
    .map(|&time| {
      let diff = if time > mean {
        time.as_nanos() as f64 - mean.as_nanos() as f64
      } else {
        mean.as_nanos() as f64 - time.as_nanos() as f64
      };
      diff * diff
    })
    .sum::<f64>() / times.len() as f64;
  
  let std_dev = Duration::from_nanos(variance.sqrt() as u64);
  
  (mean, std_dev, min, max)
}

fn benchmark_get_features(reader: &Reader, layer_idx: usize, iterations: usize, warmup_iterations: usize) -> BenchmarkResult {
  let mut feature_count = 0;
  let mut geometry_stats = GeometryStats::default();
  let transform = ScaleTransform { scale: 1.5 };

  // Count geometries once before benchmarking
  if let Ok(features) = reader.get_features(layer_idx) {
    feature_count = features.len();
    for feature in &features {
      geometry_stats.count_legacy_geometry(feature.get_geometry());
    }
  }

  // Warmup runs
  for _ in 0..warmup_iterations {
    if let Ok(features) = reader.get_features(layer_idx) {
      for feature in &features {
        apply_transform_to_legacy_geometry(feature.get_geometry(), &transform);
      }
    }
  }

  // Actual benchmark
  let mut times = Vec::with_capacity(iterations);
  for _ in 0..iterations {
    let start = Instant::now();

    if let Ok(features) = reader.get_features(layer_idx) {
      // Process all features
      for feature in &features {
        // Apply transformation manually to match the work done in get_features_iter
        apply_transform_to_legacy_geometry(feature.get_geometry(), &transform);
      }
    }

    times.push(start.elapsed());
  }

  let (mean, std_dev, min, max) = calculate_stats(&times);

  BenchmarkResult {
    method_name: "get_features",
    mean_time: mean,
    std_dev,
    min_time: min,
    max_time: max,
    feature_count,
    geometry_stats,
    iterations,
  }
}

fn benchmark_get_features_iter(
  reader: &Reader,
  layer_idx: usize,
  iterations: usize,
  warmup_iterations: usize,
) -> BenchmarkResult {
  let mut feature_count = 0;
  let mut geometry_stats = GeometryStats::default();
  let transform = ScaleTransform { scale: 1.5 };

  // Count geometries once before benchmarking
  if let Some(features) = reader.get_features_iter::<FlatCoordinateStorage, _>(layer_idx, transform)
  {
    for feature in features {
      feature_count += 1;
      for geom_result in feature.geometry {
        if let Ok(geom) = geom_result {
          geometry_stats.count_iter_geometry(&geom);
        }
      }
    }
  }

  // Warmup runs
  for _ in 0..warmup_iterations {
    if let Some(features) =
      reader.get_features_iter::<FlatCoordinateStorage, _>(layer_idx, transform)
    {
      for feature in features {
        for geom_result in feature.geometry {
          if let Ok(geom) = geom_result {
            match &geom {
              Geometry::Point { x, y } => {
                let _ = (x, y);
              }
              Geometry::LineString(coords) => {
                let _ = coords.transformed_as_slice();
              }
              Geometry::Polygon { exterior, holes } => {
                let _ = exterior.transformed_as_slice();
                for hole in holes {
                  let _ = hole.transformed_as_slice();
                }
              }
              Geometry::MultiPoint(coords) => {
                let _ = coords.transformed_as_slice();
              }
              Geometry::MultiLineString(linestrings) => {
                for ls in linestrings {
                  let _ = ls.transformed_as_slice();
                }
              }
              Geometry::MultiPolygon(polygons) => {
                for (exterior, holes) in polygons {
                  let _ = exterior.transformed_as_slice();
                  for hole in holes {
                    let _ = hole.transformed_as_slice();
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  // Actual benchmark
  let mut times = Vec::with_capacity(iterations);
  for _ in 0..iterations {
    let start = Instant::now();

    if let Some(features) =
      reader.get_features_iter::<FlatCoordinateStorage, _>(layer_idx, transform)
    {
      for feature in features {
        // Process all geometries
        for geom_result in feature.geometry {
          if let Ok(geom) = geom_result {
            // Access transformed coordinates to ensure transformation is applied
            match &geom {
              Geometry::Point { x, y } => {
                let _ = (x, y);
              }
              Geometry::LineString(coords) => {
                let _ = coords.transformed_as_slice();
              }
              Geometry::Polygon { exterior, holes } => {
                let _ = exterior.transformed_as_slice();
                for hole in holes {
                  let _ = hole.transformed_as_slice();
                }
              }
              Geometry::MultiPoint(coords) => {
                let _ = coords.transformed_as_slice();
              }
              Geometry::MultiLineString(linestrings) => {
                for ls in linestrings {
                  let _ = ls.transformed_as_slice();
                }
              }
              Geometry::MultiPolygon(polygons) => {
                for (exterior, holes) in polygons {
                  let _ = exterior.transformed_as_slice();
                  for hole in holes {
                    let _ = hole.transformed_as_slice();
                  }
                }
              }
            }
          }
        }
      }
    }

    times.push(start.elapsed());
  }

  let (mean, std_dev, min, max) = calculate_stats(&times);

  BenchmarkResult {
    method_name: "get_features_iter",
    mean_time: mean,
    std_dev,
    min_time: min,
    max_time: max,
    feature_count,
    geometry_stats,
    iterations,
  }
}

fn print_results(
  file_path: &PathBuf,
  file_size: u64,
  layer_name: &str,
  results: &[BenchmarkResult],
) {
  println!("\n=== Benchmark Results ===");
  println!("File: {}", file_path.display());
  println!("File size: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
  println!("Layer: {}", layer_name);
  println!("Mode: {}", if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });

  if results.is_empty() {
    println!("No results to display");
    return;
  }

  let first_result = &results[0];
  println!("\nData Statistics:");
  println!("  Total features: {}", first_result.feature_count);
  println!(
    "  Total geometries: {}",
    first_result.geometry_stats.total_geometries()
  );
  println!("  - Points: {}", first_result.geometry_stats.points);
  println!(
    "  - LineStrings: {}",
    first_result.geometry_stats.linestrings
  );
  println!("  - Polygons: {}", first_result.geometry_stats.polygons);
  println!(
    "    - Exterior rings: {}",
    first_result.geometry_stats.exterior_rings
  );
  println!(
    "    - Interior rings (holes): {}",
    first_result.geometry_stats.interior_rings
  );

  for result in results {
    println!("\n--- {} ---", result.method_name);
    println!("Iterations: {}", result.iterations);
    println!(
      "Mean time: {:.2} ms (±{:.2} ms)",
      result.mean_time.as_secs_f64() * 1000.0,
      result.std_dev.as_secs_f64() * 1000.0
    );
    println!(
      "Min/Max: {:.2} ms / {:.2} ms",
      result.min_time.as_secs_f64() * 1000.0,
      result.max_time.as_secs_f64() * 1000.0
    );
    println!("Time per feature: {:.2} μs", result.time_per_feature_us());
    println!("Time per geometry: {:.2} μs", result.time_per_geometry_us());
  }

  if results.len() >= 2 {
    let legacy_time = results[0].mean_time.as_secs_f64();
    let iter_time = results[1].mean_time.as_secs_f64();
    let improvement = ((legacy_time - iter_time) / legacy_time) * 100.0;

    println!("\n=== Performance Comparison ===");
    if improvement > 0.0 {
      println!("get_features_iter is {:.1}% faster than get_features", improvement);
      println!("Speed-up factor: {:.2}x", legacy_time / iter_time);
    } else {
      println!("get_features is {:.1}% faster than get_features_iter", -improvement);
      println!("Speed-up factor: {:.2}x", iter_time / legacy_time);
    }
  }
}

fn determine_iterations(file_size: u64) -> (usize, usize) {
  // Determine iterations based on file size
  let (iterations, warmup) = match file_size {
    0..=500_000 => (20, 5),           // < 500KB
    500_001..=1_000_000 => (15, 3),   // 500KB - 1MB
    1_000_001..=3_000_000 => (10, 2), // 1MB - 3MB
    3_000_001..=5_000_000 => (5, 1),  // 3MB - 5MB
    _ => (3, 1),                       // > 5MB
  };
  (iterations, warmup)
}

#[test]
fn benchmark_large_files() {
  // Test with large files
  let test_files = vec![
    // Small file for comparison
    ("mvt-fixtures/real-world/bangkok/12-3188-1888.mvt", "Small"),
    // Medium files
    (
      "mvt-fixtures/real-world/osm-qa-astana/12-2860-1369.mvt",
      "Medium (325KB)",
    ),
    (
      "mvt-fixtures/real-world/osm-qa-astana/12-2860-1367.mvt",
      "Medium (726KB)",
    ),
    // Large files
    (
      "mvt-fixtures/real-world/osm-qa-astana/12-2860-1368.mvt",
      "Large (1.0MB)",
    ),
    (
      "mvt-fixtures/real-world/osm-qa-montevideo/12-1407-2471.mvt",
      "Large (1.5MB)",
    ),
    // Very large files
    (
      "mvt-fixtures/real-world/osm-qa-montevideo/12-1408-2472.mvt",
      "Very Large (2.8MB)",
    ),
    (
      "mvt-fixtures/real-world/osm-qa-montevideo/12-1409-2471.mvt",
      "Very Large (4.9MB)",
    ),
    (
      "mvt-fixtures/real-world/osm-qa-montevideo/12-1408-2471.mvt",
      "Very Large (6.7MB)",
    ),
  ];

  for (file_path_str, size_category) in test_files {
    println!("\n\n========== {} ==========", size_category);

    let file_path = PathBuf::from(file_path_str);

    if !file_path.exists() {
      println!("Skipping non-existent file: {}", file_path.display());
      continue;
    }

    let bytes = match read(&file_path) {
      Ok(bytes) => bytes,
      Err(e) => {
        println!("Failed to read {}: {}", file_path.display(), e);
        continue;
      }
    };

    let file_size = bytes.len() as u64;
    let (iterations, warmup_iterations) = determine_iterations(file_size);

    let reader = match Reader::new(bytes) {
      Ok(reader) => reader,
      Err(e) => {
        println!("Failed to parse {}: {:?}", file_path.display(), e);
        continue;
      }
    };

    let layer_names = match reader.get_layer_names() {
      Ok(names) => names,
      Err(e) => {
        println!("Failed to get layer names: {:?}", e);
        continue;
      }
    };

    // Benchmark the first layer with features
    for (layer_idx, layer_name) in layer_names.iter().enumerate() {
      // Check if layer has features
      if let Ok(features) = reader.get_features(layer_idx) {
        if features.is_empty() {
          continue;
        }

        // Run benchmarks
        let mut results = Vec::new();

        // Benchmark get_features
        println!("Benchmarking get_features ({} iterations, {} warmup)...", iterations, warmup_iterations);
        results.push(benchmark_get_features(&reader, layer_idx, iterations, warmup_iterations));

        // Clear CPU cache
        let cache_clear_size = 20 * 1024 * 1024; // 20MB for large files
        let mut cache_clear: Vec<u8> = vec![0; cache_clear_size];
        for i in (0..cache_clear_size).step_by(64) {
          cache_clear[i] = (i % 256) as u8;
        }
        let _: u64 = cache_clear.iter().map(|&x| x as u64).sum();

        // Benchmark get_features_iter
        println!("Benchmarking get_features_iter ({} iterations, {} warmup)...", iterations, warmup_iterations);
        results.push(benchmark_get_features_iter(&reader, layer_idx, iterations, warmup_iterations));

        // Print results
        print_results(&file_path, file_size, layer_name, &results);

        // Only benchmark the first layer with features
        break;
      }
    }
  }
}
