const { Reader } = require('mvt-reader')
const { readFileSync } = require('fs')

describe('Test fixture', () => {
  test('032 parsed successfully', async () => {
    const reader = new Reader(readFileSync('mvt-fixtures/fixtures/032/tile.mvt'))
    expect(reader.getLayerNames()).toContain('hello')
    console.log(reader.getFeatures(0))
  })
})