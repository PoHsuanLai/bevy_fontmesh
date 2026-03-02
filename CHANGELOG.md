# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-02

### Changed - BREAKING

- **Generic material support**: `FontMeshPlugin`, `TextMeshBundle`, and `TextMeshGlyphsBundle` are now generic over `M: Material` (defaulting to `StandardMaterial`). Existing code using `StandardMaterial` continues to work — just replace `FontMeshPlugin` with `FontMeshPlugin::<StandardMaterial>::default()`.
- For per-glyph entities with a custom material type, add the plugin for that type: `app.add_plugins(FontMeshPlugin::<MyMaterial>::default())`.

## [0.2.2] - 2026-03-02

### Fixed

- **Side faces now visible**: Updated fontmesh dependency to 0.4.1 which fixes side face normals on 3D extruded glyphs. Previously sides were invisible due to inward-facing normals being culled by Bevy's renderer.

### Added

- `showcase` example: metallic "BEVY" text with per-glyph animated materials and orbiting camera

## [0.2.1] - 2026-02-26

### Changed
- Updated fontmesh dependency to 0.4.0 (pure functions API, parameter validation)
- Dual licensed MIT/Apache-2.0

## [0.2.0] - 2025

### Changed - BREAKING
- Updated to Bevy 0.18
- Updated to fontmesh 0.3.4 (pure functions API)

## [0.1.5] - 2024

### Changed
- Reduced Bevy feature dependencies (only `bevy_asset`, `bevy_pbr`, `bevy_render` required)
- Code refactor and cleanup

### Added
- Per-glyph entity support via `TextMeshGlyphs` and `TextMeshGlyphsBundle`
- Per-character materials and animation support

## [0.1.0] - 2024

### Added
- Initial release
- `FontMeshPlugin` for Bevy 0.17
- `TextMesh` component and `TextMeshBundle` for spawning 3D text meshes
- `TextMeshStyle` with configurable depth, anchor, and subdivision quality
- `TextAnchor` with 9 preset anchor points and `Custom(Vec2)` support
- `JustifyText` for Left, Center, Right alignment
- Font metrics utilities: `glyph_metrics`, `font_metrics`, `text_width`, `char_positions`
- TrueType (`.ttf`) and OpenType with TrueType outlines (`.otf`) support
