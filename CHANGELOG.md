# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-05-21

### Changed - BREAKING

- **Font loading now uses Bevy's standard [`bevy::text::Font`] asset.** The custom `FontMesh` asset has been removed. `TextMesh::font` and `TextMeshGlyphs::font` are now `Handle<bevy::text::Font>`, so the same handle can drive 2D UI text and 3D text meshes.
- **Text shaping now goes through [`cosmic-text`]**, the same shaper Bevy's UI text uses. You get real kerning, ligatures, BiDi, and complex-script support. Per-line `JustifyText` alignment is applied by cosmic-text; line wrapping is intentionally disabled (use `\n` for line breaks).
- **OpenType CFF/PostScript outlines are now supported.** Previously the plugin only handled TrueType outlines because of the `ttf-parser` limitation; the new `skrifa`-based fontmesh handles both.
- **Per-glyph mesh cache.** Repeated glyphs (e.g. every `'e'` in a paragraph) share one `Handle<Mesh>` instead of being re-tessellated. The cache invalidates automatically when a font asset is modified or unloaded.

### Migration Guide

The hot path is the same — the only thing that changes is the font handle type:

```rust
// 0.3
font: asset_server.load::<FontMesh>("fonts/font.ttf"),

// 0.4
font: asset_server.load::<bevy::text::Font>("fonts/font.ttf"),
// (or just `asset_server.load("fonts/font.ttf")` — the type is inferred)
```

If you set `StandardMaterial` for text, add `double_sided: true, cull_mode: None` so the back face is visible when looking through hole-punches (e.g. into the counter of a 'B'). The mesh is single-sided (matching the previous release and `ttf2mesh`).

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

## [0.1.7] - 2026-03-02

### Changed - BREAKING

- **Generic material support** (Bevy 0.17): `FontMeshPlugin`, `TextMeshBundle`, and `TextMeshGlyphsBundle` are now generic over `M: Material` (defaulting to `StandardMaterial`). Replace `FontMeshPlugin` with `FontMeshPlugin::<StandardMaterial>::default()`. For per-glyph entities with a custom material, add `FontMeshPlugin::<MyMaterial>::default()`.

## [0.1.6] - 2026-03-02

### Fixed

- **Side faces now visible** (Bevy 0.17): Bumped fontmesh to 0.3.4 which fixes side face normals on 3D extruded glyphs. Previously sides were invisible due to inward-facing normals being culled by Bevy's renderer.

## [0.1.5] - 2025

### Changed
- Reduced Bevy feature dependencies (only `bevy_asset`, `bevy_pbr`, `bevy_render` required)
- Code refactor and cleanup

### Added
- Per-glyph entity support via `TextMeshGlyphs` and `TextMeshGlyphsBundle`
- Per-character materials and animation support

## [0.1.0] - 2025

### Added
- Initial release
- `FontMeshPlugin` for Bevy 0.17
- `TextMesh` component and `TextMeshBundle` for spawning 3D text meshes
- `TextMeshStyle` with configurable depth, anchor, and subdivision quality
- `TextAnchor` with 9 preset anchor points and `Custom(Vec2)` support
- `JustifyText` for Left, Center, Right alignment
- Font metrics utilities: `glyph_metrics`, `font_metrics`, `text_width`, `char_positions`
- TrueType (`.ttf`) and OpenType with TrueType outlines (`.otf`) support
