# mvt-reader

<div align="center">
  <strong>Mapbox vector tile reader in Rust</strong>
</div>
<div align="center">
  A library for decoding and reading mapbox vector tiles in Rust and WebAssembly
</div>
<br>
<div align="center">
  <a href="https://github.com/codeart1st/mvt-reader/actions/workflows/ci.yml">
    <img src="https://github.com/codeart1st/mvt-reader/actions/workflows/ci.yml/badge.svg" alt="Build status"/>
  </a>
  <a href="https://github.com/codeart1st/mvt-reader/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/codeart1st/mvt-reader" alt="MIT license"/>
  </a>
</div>

## Features

- Decodes and reads Mapbox vector tiles in Rust
- Provides an API for accessing layer names and features within a vector tile
- Can be used as a WebAssembly module in JavaScript (enabled by the `wasm` feature)

## Build the project

```sh
cargo build --release
```

## Run tests

```sh
cargo test
wasm-pack build --release --target nodejs -d pkg/node -- --features wasm && npm test
```

## Usage

To use the `mvt-reader` library in your Rust project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
mvt-reader = "2.1.0"
```

Then, you can import and use the library in your code:

```rust
use mvt_reader::{Reader, ParserError};

fn main() -> Result<(), ParserError> {
  // Read a vector tile from file or data
  let data = vec![/* Vector tile data */];
  let reader = Reader::new(data)?;

  // Get layer names
  let layer_names = reader.get_layer_names()?;
  for name in layer_names {
    println!("Layer: {}", name);
  }

  // Get features for a specific layer
  let layer_index = 0;
  let features = reader.get_features(layer_index)?;
  for feature in features {
    todo!()
  }

  Ok(())
}
```

## WebAssembly Usage
To use the mvt-reader library as a WebAssembly module in JavaScript, you can install it with npm and use it in your JavaScript code:

```js
const { Reader } = require('mvt-reader')
const fs = require('fs')

// Example usage
const reader = new Reader(fs.readFileSync('path/to/tile.mvt'))
const layerNames = reader.getLayerNames()
console.log(layerNames)

// More code...
```

## License

This project is licensed under the [MIT License](LICENSE).

