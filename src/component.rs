use bevy::prelude::*;
use bevy::text::Font;

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
/// // `TextMesh` requires `Mesh3d` (and transitively `Transform`/`Visibility`),
/// // so it can be spawned on its own; add a `MeshMaterial3d<M>` to control how
/// // it renders.
/// commands.spawn(TextMesh {
///     text: "Hello, World!".to_string(),
///     font: asset_server.load("fonts/font.ttf"),
///     style: TextMeshStyle {
///         depth: 0.5,
///         subdivision: 25,
///         anchor: TextAnchor::Center,
///         justify: JustifyText::Center,
///     },
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
/// commands.spawn(TextMesh {
///     text: "Line 1\nLine 2\nLine 3".to_string(),
///     font: asset_server.load("fonts/font.ttf"),
///     ..default()
/// });
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
#[require(Mesh3d)]
pub struct TextMesh {
    /// The text to display. Use `\n` for line breaks.
    pub text: String,
    /// Handle to the font asset (TTF or OTF file).
    pub font: Handle<Font>,
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
/// commands.spawn((
///     TextMeshGlyphs {
///         text: "Hello".to_string(),
///         font: asset_server.load("fonts/font.ttf"),
///         style: TextMeshStyle::default(),
///     },
///     // Default material for all glyphs (can be overridden per-glyph)
///     MeshMaterial3d(materials.add(StandardMaterial::default())),
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct TextMeshGlyphs {
    /// The text to display. Use `\n` for line breaks.
    pub text: String,
    /// Handle to the font asset (TTF or OTF file).
    pub font: Handle<Font>,
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

/// Render the entity's text mesh at a constant *screen-pixel* size,
/// independent of camera distance / zoom.
///
/// Each frame the [`scale_screen_size`](crate::system::scale_screen_size)
/// system rewrites the entity's `Transform.scale` (uniform on all axes)
/// so that one line of text covers `pixel_height` pixels in the active
/// camera's render target. Position stays whatever you put it at; only
/// scale changes.
///
/// Works with both orthographic and perspective cameras and with cameras
/// rendering to either a window or a `RenderTarget::Image`. For a
/// perspective camera the system measures `world_per_pixel` at the
/// entity's depth, so screen-stable size holds wherever the entity sits
/// in front of the lens.
///
/// # Caveats
///
/// - Rewrites the *entity's* scale, so don't keep your own per-frame
///   scale tweaks on the same `Transform.scale` field — they'll be
///   overwritten. Apply scale on a parent or a child instead.
/// - Children of a `ScreenSize` entity inherit its scale, which means
///   their *positions* under the parent's local frame also scale. Put
///   ScreenSize on leaf label entities, not on a parent intended to
///   carry world-position layout.
/// - Picks the first camera it finds when there are multiple. Filter
///   with [`ScreenSizeCamera`] if you need to pin to a specific one.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct ScreenSize {
    /// Target line-height in logical pixels of the camera's render
    /// target.
    pub pixel_height: f32,
}

impl Default for ScreenSize {
    fn default() -> Self {
        Self { pixel_height: 14.0 }
    }
}

/// Marker on a `Camera` to opt that camera into being the one
/// `ScreenSize` measures against. If no entity in the world has this
/// marker, the screen-size system falls back to whichever camera
/// `Query<&Camera>` returns first — fine for single-camera apps.
#[derive(Component, Reflect, Clone, Copy, Debug, Default)]
#[reflect(Component)]
pub struct ScreenSizeCamera;
