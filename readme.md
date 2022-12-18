
# build

```bash
cargo build && target/debug/cloudtiles --max-zoom 3 convert tiles/maptiler-osm-2017-07-03-v3.6.1-germany_berlin.mbtiles tiles/berlin.cloudtiles

# cargo instruments --all-features -t "CPU Profiler" -- --max-zoom 3 convert tiles/maptiler-osm-2017-07-03-v3.6.1-germany_berlin.mbtiles tiles/berlin.cloudtiles
```

# format

<p align="center">
<img src="docs/file_format.svg?raw=true">
</p>

- integers are stored with little endian byte order
- strings are stored with utf8 encoding

## file_header (256 bytes)

| offset | length | type   | description                           |
| ------ | ------ | ------ | ------------------------------------- |
| 0      | 30     | string | `"OpenCloudTiles/Container/v1   "`    |
| 30     | 1      | u8     | `tile_format`                         |
| 31     | 1      | u8     | `tile_precompression`                 |
| 32     | 96     | blob   | empty space, fill with zeros          |
| 128    | 8      | u64    | `offset` of `meta` in the file        |
| 136    | 8      | u64    | `length` of `meta`                    |
| 144    | 8      | u64    | `offset` of `block_index` in the file |
| 152    | 8      | u64    | `length` of `block_index`             |
| 160    | 96     | blob   | empty space, fill with zeros          |

`tile_format` values:
  - `0`: png
  - `1`: jpg
  - `2`: webp
  - `16`: pbf

`tile_precompression` values:
  - `0`: uncompressed
  - `1`: gzip compressed
  - `2`: brotli compressed

## `meta`

- content of `tiles.json`
- compressed with $tile_precompression

## `block`

- one block contains tile data of up to 256x256 tiles
- so it's like a "super tile"
- it starts with concatenated, precompressed tile data and ends with a `tile_index`

## `block_index`

- brotli compressed data structure
- offsets are relative to the start of file
- empty blocks are not stored
- for each block there is a 48 byte long record:

| offset    | length | type | description                          |
| --------- | ------ | ---- | ------------------------------------ |
| 0 + 48*i  | 8      | u64  | `level`                              |
| 8 + 48*i  | 8      | u64  | `column`/256                         |
| 16 + 48*i | 8      | u64  | `row`/256                            |
| 24 + 48*i | 8      | u64  | `offset` of `block` in the file      |
| 32 + 48*i | 8      | u64  | `offset` of `tile_index` in the file |
| 40 + 48*i | 8      | u64  | `length` of `tile_index`             |

## `tile_index`

<p align="center">
<img src="docs/block_tiles.svg?raw=true">
</p>

- brotli compressed data structure
- offsets are relative to start of `block`
- identical `tile`s can be stored once and referenced multiple times
- if a tile does not exist, the length of `tile` is zero
- tiles are read horizontally then vertically
  index = (row - min_row)*(max_col - min_col + 1) + (col - min_col);

| offset    | length | type | description           |
| --------- | ------ | ---- | --------------------- |
| 0         | 1      | u8   | min_col               |
| 1         | 1      | u8   | max_col               |
| 2         | 1      | u8   | min_row               |
| 3         | 1      | u8   | max_row               |
| 4 + 12*i  | 8      | u64  | offset of tile_blob i |
| 12 + 12*i | 4      | u32  | length of tile_blob i |
