# bevy_fontmesh

[![CI](https://github.com/PoHsuanLai/bevy_fontmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/PoHsuanLai/bevy_fontmesh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/bevy_fontmesh.svg)](https://crates.io/crates/bevy_fontmesh)
[![Documentation](https://docs.rs/bevy_fontmesh/badge.svg)](https://docs.rs/bevy_fontmesh)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A simple Bevy plugin for generating 3D text meshes from fonts. Powered by [fontmesh](https://crates.io/crates/fontmesh).

<p align="center">
  <img src="images/demo.gif" alt="bevy_fontmesh demo" />
</p>

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
bevy_fontmesh = "0.1"
```

Basic usage:

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
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            style: TextMeshStyle {
                depth: 0.5,
                subdivision: 20,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
            },
        },
        material: MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.3),
            ..default()
        })),
        ..default()
    });
}
```

## Text Anchoring

Control where the text is positioned relative to its transform:

```rust
TextMeshStyle {
    anchor: TextAnchor::TopLeft,      // Top-left corner
    anchor: TextAnchor::Center,        // Center of text bounds
    anchor: TextAnchor::BottomRight,   // Bottom-right corner
    anchor: TextAnchor::Custom(Vec2::new(0.5, 0.0)), // Custom pivot (0-1 range)
    ..default()
}
```

## Text Justification

Control alignment for multiline text:

```rust
TextMesh {
    text: "Line 1\nLine 2\nLine 3".to_string(),
    style: TextMeshStyle {
        justify: JustifyText::Left,    // Left-aligned
        justify: JustifyText::Center,   // Centered
        justify: JustifyText::Right,    // Right-aligned
        ..default()
    },
    ..default()
}
```

## Styling Parameters

```rust
TextMeshStyle {
    depth: 0.5,         // Extrusion depth (0.0 for flat text)
    subdivision: 20,    // Curve quality (higher = smoother, more vertices)
    anchor: TextAnchor::Center,
    justify: JustifyText::Left,
}
```

- **depth**: Controls the Z-depth of the extruded mesh. Use `0.0` for flat 2D-style text.
- **subdivision**: Number of segments used to approximate curves. Higher values produce smoother glyphs but increase vertex count.

## Examples

Run the included examples:

```bash
# Basic 3D text
cargo run --example basic

# Multiline text with anchoring
cargo run --example multiline

# Text justification demo
cargo run --example justification

# All anchor points visualized
cargo run --example anchors

# Performance stress test (100 text meshes)
cargo run --release--example stress_test
```

## Limitations

I try not to overengineer this plugin. It does **not** provide:

- Text kerning or ligatures
- Right-to-left (RTL) text layout
- Dynamic text effects (shadows, outlines)
- 2D text rendering
- Text wrapping by width

For advanced typography needs, consider implementing additional logic on top of the generated meshes.

## Bevy Version Compatibility

| bevy_fontmesh | Bevy |
|---------------|------|
| 0.1           | 0.17 |

## License

MIT License - see [LICENSE](LICENSE) for details.
