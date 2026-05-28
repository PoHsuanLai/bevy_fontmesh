//! A Bevy plugin for generating 3D text meshes from fonts.
//!
//! Starting with 0.4 the plugin uses Bevy's standard [`bevy::text::Font`] asset
//! for font loading and `cosmic-text` for shaping, so the same `Handle<Font>`
//! you use for 2D UI text also drives 3D text meshes. This brings real text
//! shaping (kerning, ligatures, BiDi, complex scripts) and full OpenType/CFF
//! support.
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
//!     commands.spawn((
//!         TextMesh {
//!             text: "Hello!".to_string(),
//!             font: asset_server.load("fonts/font.ttf"),
//!             ..default()
//!         },
//!         MeshMaterial3d(materials.add(StandardMaterial::default())),
//!     ));
//! }
//! ```
//!
//! # Custom Materials
//!
//! Add `FontMeshPlugin` a second time with your material type:
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
//! # Font Format Support
//!
//! Both TrueType (`.ttf`) and OpenType (`.otf`, including CFF/PostScript
//! outlines) are supported via `skrifa`. Loading is handled by Bevy's
//! standard font loader.

mod component;
pub mod prelude;
mod system;

pub use component::{
    GlyphMesh, JustifyText, ScreenSize, ScreenSizeCamera, TextAnchor, TextMesh, TextMeshGlyphs,
    TextMeshStyle,
};
pub use system::{
    on_font_asset_event, scale_screen_size, update_glyph_meshes, FontMeshRegistry, GlyphMeshCache,
    TextMeshComputed, TextMeshGlyphsComputed,
};

use bevy::prelude::*;
use std::marker::PhantomData;
use system::update_text_meshes;

/// Shared one-time setup so that multiple [`FontMeshPlugin`] instances with
/// different material types don't double-register systems and resources.
struct SharedFontMeshPlugin;

impl Plugin for SharedFontMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontMeshRegistry>()
            .init_resource::<GlyphMeshCache>()
            .register_type::<TextMesh>()
            .register_type::<TextMeshGlyphs>()
            .register_type::<TextMeshStyle>()
            .register_type::<TextAnchor>()
            .register_type::<JustifyText>()
            .register_type::<GlyphMesh>()
            .register_type::<ScreenSize>()
            .register_type::<ScreenSizeCamera>()
            .add_systems(Update, update_text_meshes)
            .add_systems(Update, on_font_asset_event)
            // PostUpdate: read this frame's GlobalTransform, write to local
            // Transform; propagation picks the change up next frame.
            .add_systems(PostUpdate, scale_screen_size);
    }
}

/// Plugin that enables 3D text mesh generation from fonts.
///
/// Generic over the material type `M`. Defaults to [`StandardMaterial`].
pub struct FontMeshPlugin<M: Material = StandardMaterial>(PhantomData<M>);

impl<M: Material> Default for FontMeshPlugin<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M: Material> Plugin for FontMeshPlugin<M> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SharedFontMeshPlugin>() {
            app.add_plugins(SharedFontMeshPlugin);
        }
        app.add_systems(Update, update_glyph_meshes::<M>);
    }
}
