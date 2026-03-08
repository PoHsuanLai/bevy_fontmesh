# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2026-03-08

### Fixed

- **fontmesh 0.3.4 compatibility**: fontmesh 0.3.4 replaced the `Font` struct with a stateless `Face` + free-function API (`parse_font`, `char_to_mesh_3d`, `glyph_advance`, etc.). Versions 0.1.6 and 0.1.7 declared `fontmesh = "^0.3"` but called the old pre-0.3.4 API, causing 28 compile errors. All call sites are now updated to the correct fontmesh 0.3.4 API.

### Removed

- **`ParsedFontCache` resource**: Removed the font cache that used `unsafe` to forge `'static` lifetimes. Fonts are now parsed on-demand via `fontmesh::Face::parse()` (cheap — just reads the table directory). This eliminates unsound lifetime handling.
- **`cleanup_font_cache` system**: No longer needed without the cache.

### Changed

- Refactored `update_text_meshes` and `update_glyph_meshes` to share layout logic via extracted `layout_glyphs` and `combine_glyph_meshes` helpers.
- `generate_glyph_mesh` now takes `&fontmesh::Face` instead of `&fontmesh::Font`.

## [0.1.7] - 2026-03-02

### Changed - BREAKING

- **Generic material support**: `FontMeshPlugin`, `TextMeshBundle`, and `TextMeshGlyphsBundle` are now generic over `M: Material` (defaulting to `StandardMaterial`). Replace `FontMeshPlugin` with `FontMeshPlugin::<StandardMaterial>::default()`. For per-glyph entities with a custom material, add `FontMeshPlugin::<MyMaterial>::default()`.

## [0.1.6] - 2026-03-02

### Fixed

- **Side faces now visible**: Bumped fontmesh to 0.3.4 which fixes side face normals on 3D extruded glyphs. Previously sides were invisible due to inward-facing normals being culled by Bevy's renderer.

## [0.1.5] - 2025

- Bevy 0.17 support (maintenance)

## [0.1.4] - 2025

- Per-glyph entity support via `TextMeshGlyphs`

## [0.1.3] - 2025

- Previous stable release
