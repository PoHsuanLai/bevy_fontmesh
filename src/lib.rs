//! A simple Bevy plugin for generating 3D text meshes from fonts.
//!
//! This plugin provides an easy way to create 3D text geometry in Bevy applications.
//! It handles only mesh generation, leaving materials, lighting, transforms, and
//! rendering to Bevy's standard systems.
//!
//! # Quick Start
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_fontmesh::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(FontMeshPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(
//!     mut commands: Commands,
//!     asset_server: Res<AssetServer>,
//!     mut materials: ResMut<Assets<StandardMaterial>>,
//! ) {
//!     commands.spawn(TextMeshBundle {
//!         text_mesh: TextMesh {
//!             text: "Hello!".to_string(),
//!             font: asset_server.load("fonts/font.ttf"),
//!             ..default()
//!         },
//!         material: MeshMaterial3d(materials.add(StandardMaterial::default())),
//!         ..default()
//!     });
//! }
//! ```
//!
//! # Features
//!
//! - Generates 3D mesh geometry from TrueType fonts
//! - Supports multiline text with `\n` line breaks
//! - Configurable text anchoring (9 presets + custom pivot points)
//! - Text justification (left, center, right)
//! - Adjustable extrusion depth and curve subdivision
//! - Automatic mesh regeneration when text or style changes
//!
//! # Font Format Support
//!
//! - TrueType (`.ttf`) fonts are fully supported
//! - OpenType (`.otf`) fonts with TrueType outlines work
//! - OpenType fonts with CFF/PostScript outlines are not supported (ttf-parser limitation)

mod asset;
mod component;
pub mod prelude;
mod system;

pub use asset::{FontMesh, FontMetrics, GlyphMetrics};
pub use component::{
    GlyphMesh, JustifyText, TextAnchor, TextMesh, TextMeshBundle, TextMeshGlyphs,
    TextMeshGlyphsBundle, TextMeshStyle,
};
pub use system::{generate_glyph_mesh, TextMeshComputed, TextMeshGlyphsComputed};

use asset::FontMeshLoader;
use bevy::prelude::*;
use system::{update_glyph_meshes, update_text_meshes};

/// Plugin that enables 3D text mesh generation from fonts.
///
/// This plugin registers the necessary assets, loaders, and systems to automatically
/// generate 3D mesh geometry from [`TextMesh`] components. Simply add this plugin to
/// your Bevy app and spawn entities with [`TextMeshBundle`].
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_fontmesh::FontMeshPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(FontMeshPlugin)
///     .run();
/// ```
///
/// The plugin automatically:
/// - Registers the [`FontMesh`] asset type for loading TTF/OTF fonts
/// - Adds a system that generates meshes when [`TextMesh`] components are added or changed
/// - Enables reflection for [`TextMesh`] components for editor integration
pub struct FontMeshPlugin;

impl Plugin for FontMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FontMesh>()
            .init_asset_loader::<FontMeshLoader>()
            .register_type::<TextMesh>()
            .register_type::<TextMeshGlyphs>()
            .register_type::<GlyphMesh>()
            .add_systems(Update, (update_text_meshes, update_glyph_meshes));
    }
}
