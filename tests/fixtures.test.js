const { Reader } = require('mvt-reader')
const { readFileSync } = require('fs')

describe('Test fixture', () => {
  test('032 parsed successfully', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/032/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    expect(reader.getFeatures(0)).toStrictEqual([
      {
        geometry: { coordinates: [[25, 17]], type: 'MultiPoint' },
        properties: { key1: 'i am a string value' },
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