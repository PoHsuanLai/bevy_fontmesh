# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - 2026-03-02

### Fixed

- **Side faces now visible**: Bumped fontmesh to 0.3.4 which fixes side face normals on 3D extruded glyphs. Previously sides were invisible due to inward-facing normals being culled by Bevy's renderer.

## [0.1.5] - 2025

- Bevy 0.17 support (maintenance)

## [0.1.4] - 2024

- Per-glyph entity support via `TextMeshGlyphs`

## [0.1.3] - 2024

- Previous stable release
