# bevy_fontmesh

[![CI](https://github.com/PoHsuanLai/bevy_fontmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/PoHsuanLai/bevy_fontmesh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/bevy_fontmesh.svg)](https://crates.io/crates/bevy_fontmesh)
[![Documentation](https://docs.rs/bevy_fontmesh/badge.svg)](https://docs.rs/bevy_fontmesh)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A simple and focused Bevy plugin for generating 3D text meshes from fonts. Powered by [fontmesh](https://crates.io/crates/fontmesh).

<p align="center">
  <video src="https://github.com/user-attachments/assets/5a519f0e-b836-4dce-bd1b-eb2867e1437b" controls></video>
</p>

## What it does

Turns TrueType fonts into 3D meshes. You can control the extrusion depth, anchor points, and subdivision quality. Also supports per-character entities if you want to style or animate individual glyphs.

The plugin just generates the meshes - Bevy handles everything else (materials, lighting, rendering).

## Quick Start

```toml
[dependencies]
bevy = "0.17"
bevy_fontmesh = "0.1.4"
```

```rust
use bevy::prelude::*;
use bevy_fontmesh::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(TextMeshBundle {
        text_mesh: TextMesh {
            text: "Hello, World!".to_string(),
            font: asset_server.load("fonts/font.ttf"),
            style: TextMeshStyle {
                depth: 0.5,
                anchor: TextAnchor::Center,
                ..default()
            },
        },
        material: MeshMaterial3d(materials.add(StandardMaterial::default())),
        ..default()
    });
}
```

For detailed API documentation and more examples, see [docs.rs/bevy_fontmesh](https://docs.rs/bevy_fontmesh).

## Examples

```bash
cargo run --example basic                # Simple 3D text
cargo run --example multiline             # Multiline with anchoring
cargo run --example justification         # Text alignment
cargo run --example anchors               # All anchor points
cargo run --example per_glyph             # Per-character styling
cargo run --release --example stress_test # Performance test
```

## Why another text plugin?

I wanted something simple that just generates meshes and lets Bevy do the rest. No fancy features, no complex API - just font → mesh.

Supported Formats

- TrueType (`.ttf`) - fully supported
- OpenType (`.otf`) with TrueType outlines - supported
- OpenType with CFF/PostScript outlines - not supported (ttf-parser limitation)

## Bevy Version Compatibility

| bevy_fontmesh | Bevy |
| ------------- | ---- |
| 0.1.4         | 0.17 |

## License

MIT License - see [LICENSE](LICENSE) for details.
<<<<<<< HEAD
