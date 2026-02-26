# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
