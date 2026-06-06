use std::fs::read;

use mvt_reader::{Reader, error::ParserError};

#[test]
fn u8_coordinate_overflow_returns_error() {
    // Fixture 053 has coordinates (0,0), (4096,0), (4096,4096), (0,4096)
    // which clearly exceed u8 max (255)
    let bytes = read("mvt-fixtures/fixtures/053/tile.mvt").unwrap();
    let reader = Reader::new(bytes).unwrap();

    let result = reader.get_features_as::<u8>(0);
    assert!(
        matches!(result, Err(ParserError::CoordinateOverflow { .. })),
        "Expected CoordinateOverflow error for coordinates exceeding u8 range, got: {:?}",
        result
    );
}

#[test]
fn u8_coordinate_within_range_succeeds() {
    // Fixture 002 has a point at (25, 17) which fits in u8
    let bytes = read("mvt-fixtures/fixtures/002/tile.mvt").unwrap();
    let reader = Reader::new(bytes).unwrap();

    let result = reader.get_features_as::<u8>(0);
    assert!(
        result.is_ok(),
        "Expected success for coordinates within u8 range, got: {:?}",
        result
    );
}

#[test]
fn i32_coordinate_large_values_succeeds() {
    // Fixture 053 has coordinates up to 4096 which fit in i32
    let bytes = read("mvt-fixtures/fixtures/053/tile.mvt").unwrap();
    let reader = Reader::new(bytes).unwrap();

    let result = reader.get_features_as::<i32>(0);
    assert!(
        result.is_ok(),
        "Expected success for coordinates within i32 range, got: {:?}",
        result
    );
}
