use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use thiserror::Error;

/// Asset containing font data for 3D text mesh generation.
///
/// This asset type holds the raw bytes of a TrueType font file.
/// Fonts are automatically loaded from the asset server and used by the [`TextMesh`](crate::TextMesh)
/// component to generate 3D mesh geometry.
///
/// # Loading Fonts
///
/// Fonts are loaded like any other Bevy asset:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fontmesh::FontMesh;
/// # fn example(asset_server: Res<AssetServer>) {
/// let font: Handle<FontMesh> = asset_server.load("fonts/MyFont.ttf");
/// # }
/// ```
///
/// Place your font files in the `assets/fonts/` directory (or any subdirectory of `assets/`).
///
/// # Supported Formats
///
/// - TrueType (`.ttf`)
///
/// **Note**: Some OpenType fonts (`.otf`) with TrueType outlines are supported,
/// but OpenType fonts with CFF/PostScript outlines are not (limitation of ttf-parser).
#[derive(Asset, TypePath, Debug)]
pub struct FontMesh {
    /// Raw font file data in TTF or OTF format.
    pub data: Vec<u8>,
}

/// Metrics for a single glyph
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    /// Horizontal advance width (how far to move cursor after this glyph)
    pub advance: f32,
    /// Whether the glyph has visible geometry (some chars like space don't)
    pub has_outline: bool,
}

/// Font-level metrics
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Distance from baseline to top of tallest glyph
    pub ascender: f32,
    /// Distance from baseline to bottom of lowest glyph (typically negative)
    pub descender: f32,
    /// Extra space between lines
    pub line_gap: f32,
    /// Total line height (ascender - descender + line_gap)
    pub line_height: f32,
}

impl FontMesh {
    /// Get metrics for a specific character.
    ///
    /// Returns `None` if the character is not in the font.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fontmesh::FontMesh;
    /// # fn example(font_assets: Res<Assets<FontMesh>>, font_handle: Handle<FontMesh>) {
    /// if let Some(font) = font_assets.get(&font_handle) {
    ///     if let Some(metrics) = font.glyph_metrics('A') {
    ///         println!("Advance width of 'A': {}", metrics.advance);
    ///     }
    /// }
    /// # }
    /// ```
    pub fn glyph_metrics(&self, character: char) -> Option<GlyphMetrics> {
        let font = fontmesh::Font::from_bytes(&self.data).ok()?;
        let glyph = font.glyph_by_char(character).ok()?;

        Some(GlyphMetrics {
            advance: glyph.advance(),
            has_outline: glyph.outline().is_ok(),
        })
    }

    /// Get font-level metrics (ascender, descender, line height, etc.)
    ///
    /// Returns `None` if the font data is invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fontmesh::FontMesh;
    /// # fn example(font_assets: Res<Assets<FontMesh>>, font_handle: Handle<FontMesh>) {
    /// if let Some(font) = font_assets.get(&font_handle) {
    ///     if let Some(metrics) = font.font_metrics() {
    ///         println!("Line height: {}", metrics.line_height);
    ///     }
    /// }
    /// # }
    /// ```
    pub fn font_metrics(&self) -> Option<FontMetrics> {
        let font = fontmesh::Font::from_bytes(&self.data).ok()?;

        let ascender = font.ascender();
        let descender = font.descender();
        let line_gap = font.line_gap();

        Some(FontMetrics {
            ascender,
            descender,
            line_gap,
            line_height: ascender - descender + line_gap,
        })
    }

    /// Calculate the width of a text string.
    ///
    /// This sums the advance widths of all characters. Does not account for kerning.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fontmesh::FontMesh;
    /// # fn example(font_assets: Res<Assets<FontMesh>>, font_handle: Handle<FontMesh>) {
    /// if let Some(font) = font_assets.get(&font_handle) {
    ///     let width = font.text_width("Hello");
    ///     println!("Text width: {}", width);
    /// }
    /// # }
    /// ```
    pub fn text_width(&self, text: &str) -> f32 {
        let font = match fontmesh::Font::from_bytes(&self.data) {
            Ok(f) => f,
            Err(_) => return 0.0,
        };

        let mut width = 0.0;
        for ch in text.chars() {
            if let Ok(glyph) = font.glyph_by_char(ch) {
                width += glyph.advance();
            } else if ch.is_whitespace() {
                width += 0.3; // Fallback for space
            }
        }
        width
    }

    /// Get character positions for a line of text.
    ///
    /// Returns a vector of (char_index, x_position) pairs for each character.
    /// Useful for cursor positioning in text editors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fontmesh::FontMesh;
    /// # fn example(font_assets: Res<Assets<FontMesh>>, font_handle: Handle<FontMesh>) {
    /// if let Some(font) = font_assets.get(&font_handle) {
    ///     let positions = font.char_positions("Hello");
    ///     for (idx, x) in positions {
    ///         println!("Char {} at x={}", idx, x);
    ///     }
    /// }
    /// # }
    /// ```
    pub fn char_positions(&self, text: &str) -> Vec<(usize, f32)> {
        let font = match fontmesh::Font::from_bytes(&self.data) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let mut positions = Vec::with_capacity(text.len());
        let mut x = 0.0;

        for (idx, ch) in text.chars().enumerate() {
            positions.push((idx, x));

            if let Ok(glyph) = font.glyph_by_char(ch) {
                x += glyph.advance();
            } else if ch.is_whitespace() {
                x += 0.3;
            }
        }

        positions
    }
}

/// Asset loader for TrueType and OpenType font files.
///
/// This loader is registered automatically by [`FontMeshPlugin`](crate::FontMeshPlugin)
/// and handles `.ttf` and `.otf` file extensions.
#[derive(Default)]
pub struct FontMeshLoader;

/// Errors that can occur when loading font assets.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FontMeshLoaderError {
    /// Failed to read the font file from disk.
    #[error("Could not load font file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for FontMeshLoader {
    type Asset = FontMesh;
    type Settings = ();
    type Error = FontMeshLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;
        Ok(FontMesh { data })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
