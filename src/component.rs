use crate::asset::FontMesh;
use bevy::prelude::*;

/// Determines where the text mesh is positioned relative to its transform origin.
///
/// The anchor point acts as a pivot for positioning the text. For example, [`TextAnchor::Center`]
/// places the transform at the center of the text bounds, while [`TextAnchor::BottomLeft`]
/// places it at the bottom-left corner.
///
/// # Examples
///
/// ```
/// # use bevy_fontmesh::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::prelude::default;
/// // Text centered on its transform
/// let style = TextMeshStyle {
///     anchor: TextAnchor::Center,
///     ..default()
/// };
///
/// // Text positioned by its top-left corner
/// let style = TextMeshStyle {
///     anchor: TextAnchor::TopLeft,
///     ..default()
/// };
///
/// // Custom pivot point at 25% from left, 75% from bottom
/// let style = TextMeshStyle {
///     anchor: TextAnchor::Custom(Vec2::new(0.25, 0.75)),
///     ..default()
/// };
/// ```
#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq)]
pub enum TextAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    /// Custom anchor point (0.0-1.0), where (0,0) is BottomLeft and (1,1) is TopRight
    Custom(Vec2),
}

/// Component for generating 3D text meshes from fonts.
///
/// When added to an entity, this component triggers automatic generation of a 3D mesh
/// based on the specified text, font, and style. The mesh is regenerated whenever the
/// component changes.
///
/// # Examples
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::prelude::*;
/// # fn example(mut commands: Commands, asset_server: Res<AssetServer>) {
/// commands.spawn(TextMeshBundle {
///     text_mesh: TextMesh {
///         text: "Hello, World!".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         style: TextMeshStyle {
///             depth: 0.5,
///             subdivision: 25,
///             anchor: TextAnchor::Center,
///             justify: JustifyText::Center,
///         },
///     },
///     ..default()
/// });
/// # }
/// ```
///
/// # Multiline Text
///
/// Use `\n` for line breaks:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::prelude::*;
/// # fn example(mut commands: Commands, asset_server: Res<AssetServer>) {
/// commands.spawn(TextMeshBundle {
///     text_mesh: TextMesh {
///         text: "Line 1\nLine 2\nLine 3".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         ..default()
///     },
///     ..default()
/// });
/// # }
/// ```
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TextMesh {
    /// The text to display. Use `\n` for line breaks.
    pub text: String,
    /// Handle to the font asset (TTF or OTF file).
    pub font: Handle<FontMesh>,
    /// Visual style configuration for the text mesh.
    pub style: TextMeshStyle,
}

/// Controls horizontal alignment of multiline text.
///
/// This determines how multiple lines of text are aligned relative to each other.
/// For single-line text, justification has no visual effect.
///
/// # Examples
///
/// ```
/// # use bevy_fontmesh::prelude::*;
/// # use bevy::prelude::default;
/// // Left-aligned text (default)
/// let style = TextMeshStyle {
///     justify: JustifyText::Left,
///     ..default()
/// };
///
/// // Centered text
/// let style = TextMeshStyle {
///     justify: JustifyText::Center,
///     ..default()
/// };
///
/// // Right-aligned text
/// let style = TextMeshStyle {
///     justify: JustifyText::Right,
///     ..default()
/// };
/// ```
#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JustifyText {
    /// Align text to the left edge.
    #[default]
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right edge.
    Right,
}

/// Visual styling parameters for generated text meshes.
///
/// Controls the 3D extrusion depth, curve smoothness, positioning, and alignment
/// of the generated mesh geometry.
///
/// # Examples
///
/// ```
/// # use bevy_fontmesh::prelude::*;
/// # use bevy::prelude::default;
/// // Flat 2D-style text
/// let flat = TextMeshStyle {
///     depth: 0.0,
///     subdivision: 15,
///     ..default()
/// };
///
/// // Deep 3D text with high quality curves
/// let deep = TextMeshStyle {
///     depth: 1.0,
///     subdivision: 30,
///     anchor: TextAnchor::Center,
///     justify: JustifyText::Center,
/// };
///
/// // Low-poly stylized text
/// let lowpoly = TextMeshStyle {
///     depth: 0.2,
///     subdivision: 5,
///     ..default()
/// };
/// ```
#[derive(Reflect, Clone, Debug)]
pub struct TextMeshStyle {
    /// Extrusion depth of the 3D mesh.
    ///
    /// Controls how far the text is extruded in the Z direction. A value of `0.0`
    /// produces flat, 2D-style text. Higher values create more pronounced 3D geometry.
    /// The depth is measured in font units (typically relative to the font's em height).
    ///
    /// Recommended range: `0.0` to `2.0`.
    pub depth: f32,

    /// Number of segments used to approximate curved glyph outlines.
    ///
    /// Higher values produce smoother curves but increase vertex count and memory usage.
    /// Lower values create a more angular, low-poly appearance.
    ///
    /// Recommended range: `5` (low-poly) to `30` (very smooth).
    /// Default: `20`.
    pub subdivision: u8,

    /// Position of the text mesh relative to its transform origin.
    ///
    /// Determines which point of the text bounds is placed at the entity's transform position.
    /// See [`TextAnchor`] for available options.
    pub anchor: TextAnchor,

    /// Horizontal alignment for multiline text.
    ///
    /// Controls how multiple lines of text are aligned relative to each other.
    /// Has no effect on single-line text. See [`JustifyText`] for options.
    pub justify: JustifyText,
}

impl Default for TextMeshStyle {
    fn default() -> Self {
        Self {
            depth: 0.1,
            subdivision: 20, // Default low poly-ish but smooth enough
            anchor: TextAnchor::TopLeft,
            justify: JustifyText::Left,
        }
    }
}

/// Component for generating individual 3D mesh entities for each character.
///
/// Unlike [`TextMesh`] which creates a single combined mesh for all characters,
/// `TextMeshGlyphs` spawns a separate child entity for each character. This enables:
/// - Per-character materials (different colors for syntax highlighting)
/// - Per-character animations and effects
/// - Efficient updates when only some characters change
/// - Individual character picking/interaction
///
/// Each child entity will have a [`GlyphMesh`] component with its character index.
///
/// # Examples
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::prelude::*;
/// # fn example(
/// #     mut commands: Commands,
/// #     asset_server: Res<AssetServer>,
/// #     mut materials: ResMut<Assets<StandardMaterial>>,
/// # ) {
/// commands.spawn(TextMeshGlyphsBundle {
///     text_glyphs: TextMeshGlyphs {
///         text: "Hello".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         style: TextMeshStyle::default(),
///     },
///     // Default material for all glyphs (can be overridden per-glyph)
///     material: MeshMaterial3d(materials.add(StandardMaterial::default())),
///     ..default()
/// });
/// # }
/// ```
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TextMeshGlyphs {
    /// The text to display. Use `\n` for line breaks.
    pub text: String,
    /// Handle to the font asset (TTF or OTF file).
    pub font: Handle<FontMesh>,
    /// Visual style configuration for the glyph meshes.
    pub style: TextMeshStyle,
}

/// Marker component for individual glyph mesh entities.
///
/// This component is automatically added to child entities spawned by [`TextMeshGlyphs`].
/// It contains metadata about the glyph's position in the text.
///
/// # Fields
///
/// - `char_index`: The index of this character in the original text string
/// - `line_index`: The line number (0-indexed) this character appears on
/// - `character`: The actual character this glyph represents
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct GlyphMesh {
    /// Index of this character in the text string (0-indexed)
    pub char_index: usize,
    /// Line number this character is on (0-indexed)
    pub line_index: usize,
    /// The character this glyph represents
    pub character: char,
}

/// Convenience bundle for spawning 3D text with per-character entities.
///
/// This bundle is similar to [`TextMeshBundle`] but uses [`TextMeshGlyphs`] instead,
/// which spawns separate child entities for each character.
///
/// # Examples
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::prelude::*;
/// # fn example(
/// #     mut commands: Commands,
/// #     asset_server: Res<AssetServer>,
/// #     mut materials: ResMut<Assets<StandardMaterial>>,
/// # ) {
/// commands.spawn(TextMeshGlyphsBundle {
///     text_glyphs: TextMeshGlyphs {
///         text: "Code".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         style: TextMeshStyle {
///             depth: 0.1,
///             ..default()
///         },
///     },
///     material: MeshMaterial3d(materials.add(StandardMaterial {
///         base_color: Color::WHITE,
///         ..default()
///     })),
///     transform: Transform::from_xyz(0.0, 0.0, 0.0),
///     ..default()
/// });
/// # }
/// ```
#[derive(Bundle, Default)]
pub struct TextMeshGlyphsBundle {
    /// The text glyphs component that drives per-character mesh generation.
    pub text_glyphs: TextMeshGlyphs,
    /// Default material for glyph meshes (can be overridden per-glyph).
    pub material: MeshMaterial3d<StandardMaterial>,
    /// Local transform of the parent entity.
    pub transform: Transform,
    /// Global transform (computed automatically).
    pub global_transform: GlobalTransform,
    /// Visibility of the entity.
    pub visibility: Visibility,
    /// Inherited visibility (computed automatically).
    pub inherited_visibility: InheritedVisibility,
    /// View visibility (computed automatically).
    pub view_visibility: ViewVisibility,
}

/// Convenience bundle for spawning 3D text entities.
///
/// This bundle includes all necessary components for rendering 3D text in Bevy:
/// the [`TextMesh`] component for mesh generation, along with all standard 3D rendering
/// components (mesh, material, transform, visibility).
///
/// # Examples
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::prelude::*;
/// # fn example(
/// #     mut commands: Commands,
/// #     asset_server: Res<AssetServer>,
/// #     mut materials: ResMut<Assets<StandardMaterial>>,
/// # ) {
/// commands.spawn(TextMeshBundle {
///     text_mesh: TextMesh {
///         text: "Hello, Bevy!".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         style: TextMeshStyle {
///             depth: 0.5,
///             anchor: TextAnchor::Center,
///             ..default()
///         },
///     },
///     material: MeshMaterial3d(materials.add(StandardMaterial {
///         base_color: Color::srgb(1.0, 0.5, 0.2),
///         ..default()
///     })),
///     transform: Transform::from_xyz(0.0, 1.0, 0.0),
///     ..default()
/// });
/// # }
/// ```
#[derive(Bundle, Default)]
pub struct TextMeshBundle {
    /// The text mesh component that drives mesh generation.
    pub text_mesh: TextMesh,
    /// The 3D mesh handle (automatically populated by the plugin system).
    pub mesh: Mesh3d,
    /// Material applied to the text mesh.
    pub material: MeshMaterial3d<StandardMaterial>,
    /// Local transform of the entity.
    pub transform: Transform,
    /// Global transform (computed automatically).
    pub global_transform: GlobalTransform,
    /// Visibility of the entity.
    pub visibility: Visibility,
    /// Inherited visibility (computed automatically).
    pub inherited_visibility: InheritedVisibility,
    /// View visibility (computed automatically).
    pub view_visibility: ViewVisibility,
}
