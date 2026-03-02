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
//!         .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
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
//! # Custom Materials
//!
//! To use a custom material type, add `FontMeshPlugin` a second time with your material:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_fontmesh::FontMeshPlugin;
//!
//! # use bevy::render::render_resource::AsBindGroup;
//! # #[derive(Asset, TypePath, AsBindGroup, Clone)]
//! # struct MyCustomMaterial {}
//! # impl Material for MyCustomMaterial {}
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
//!     .add_plugins(FontMeshPlugin::<MyCustomMaterial>::default())
//!     .run();
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
pub use system::{
    generate_glyph_mesh, update_glyph_meshes, TextMeshComputed, TextMeshGlyphsComputed,
};

use asset::FontMeshLoader;
use bevy::prelude::*;
use std::marker::PhantomData;
use system::update_text_meshes;

/// Internal plugin that handles one-time shared setup (asset loader, reflection, text mesh system).
/// Used by [`FontMeshPlugin`] to avoid duplicate registration when multiple material types are added.
struct SharedFontMeshPlugin;

impl Plugin for SharedFontMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FontMesh>()
            .init_asset_loader::<FontMeshLoader>()
            .register_type::<TextMesh>()
            .register_type::<TextMeshGlyphs>()
            .register_type::<GlyphMesh>()
            .add_systems(Update, update_text_meshes);
    }
}

/// Plugin that enables 3D text mesh generation from fonts.
///
/// Generic over the material type `M`. Defaults to [`StandardMaterial`], so existing
/// code using `FontMeshPlugin` without a type parameter continues to work unchanged.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_fontmesh::FontMeshPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
///     .run();
/// ```
///
/// For custom materials, add the plugin again with your material type:
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_fontmesh::FontMeshPlugin;
///
/// # use bevy::render::render_resource::AsBindGroup;
/// # #[derive(Asset, TypePath, AsBindGroup, Clone)]
/// # struct MyCustomMaterial {}
/// # impl Material for MyCustomMaterial {}
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
///     .add_plugins(FontMeshPlugin::<MyCustomMaterial>::default())
///     .run();
/// ```
///
/// The plugin automatically:
/// - Registers the [`FontMesh`] asset type for loading TTF/OTF fonts (once, on first add)
/// - Adds a system that generates meshes when [`TextMesh`] components are added or changed
/// - Adds a system that generates per-glyph meshes for [`TextMeshGlyphs`] using material `M`
/// - Enables reflection for [`TextMesh`] components for editor integration
pub struct FontMeshPlugin<M: Material = StandardMaterial>(PhantomData<M>);

impl<M: Material> Default for FontMeshPlugin<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M: Material> Plugin for FontMeshPlugin<M> {
    fn build(&self, app: &mut App) {
        // Shared setup (asset loader, reflection, TextMesh system) runs only once.
        if !app.is_plugin_added::<SharedFontMeshPlugin>() {
            app.add_plugins(SharedFontMeshPlugin);
        }
        // Material-specific glyph system — one per material type.
        app.add_systems(Update, update_glyph_meshes::<M>);
    }
}
