const { Reader } = require('mvt-reader')
const { readFileSync } = require('fs')

describe('Test fixture', () => {
  test('032 contains layer metadata', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/032/tile.mvt'))
    const layers = reader.getLayerMetadata()

    expect(layers.length).toBe(1)

    const layer = layers[0]

    expect(layer.name).toBe('hello')
    expect(layer.extent).toBe(4096)
    expect(layer.version).toBe(2)
    expect(layer.feature_count).toBe(1)
    expect(layer.layer_index).toBe(0)
  })

  test('032 parsed successfully (string_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/032/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 'i am a string value' },
        type: 'Feature'
      }
    ])
  })

  test('033 parsed successfully (float_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/033/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 3.0999999046325684 },
        type: 'Feature'
      }
    ])
  })

  test('034 parsed successfully (double_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/034/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 1.23 },
        type: 'Feature'
      }
    ])
  })

  test('035 parsed successfully (int_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/035/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 6 },
        type: 'Feature'
      }
    ])
  })

  test('036 parsed successfully (uint_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/036/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 87948 },
        type: 'Feature'
      }
    ])
  })

  test('037 parsed successfully (sint_value)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/037/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: { key1: 87948 },
        type: 'Feature'
      }
    ])
  })

  test('038 parsed successfully (different value types)', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/038/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        id: 1,
        properties: {
          string_value: 'ello',
          bool_value: true,
          int_value: 6,
          double_value: 1.23,
          float_value: 3.0999999046325684,
          sint_value: -87948,
          uint_value: 87948
        },
        type: 'Feature'
      }
    ])
  })

  test('013 failed successfully', async () => {
    let error
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/013/tile.mvt'), e => {
      error = e
    })
    expect(reader).toBeDefined()
    expect(error).toBe('ParserError { source: DecodeError { source: DecodeError { description: \"invalid wire type: Varint (expected LengthDelimited)\", stack: [(\"Layer\", \"keys\"), (\"Tile\", \"layers\")] } } }')

    expect(reader.getLayerNames()).toBeNull()
    expect(reader.getFeatures(0)).toBeNull()
  })
})