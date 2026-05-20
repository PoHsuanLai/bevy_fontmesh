//! Tests for font metric helpers exposed via the underlying `fontmesh` crate.
//!
//! In 0.4 we use `bevy_text::Font` as the canonical font asset, so all the
//! ergonomic helpers that used to live on `FontMesh` are now applied by
//! callers via `fontmesh::*` and `cosmic_text`. These tests pin the parts of
//! that contract that bevy_fontmesh relies on internally.

use fontmesh::{ascender, descender, glyph_advance, glyph_id, line_gap, parse_font};
use std::fs;

fn font_bytes() -> Vec<u8> {
    fs::read("assets/fonts/FiraMono-Medium.ttf")
        .expect("Failed to load test font - make sure assets/fonts/FiraMono-Medium.ttf exists")
}

#[test]
fn test_font_metrics_api() {
    let bytes = font_bytes();
    let font = parse_font(&bytes).expect("parse_font");

    let asc = ascender(&font);
    let desc = descender(&font);
    let gap = line_gap(&font);

    assert!(asc > 0.0, "Ascender should be positive");
    assert!(desc < 0.0, "Descender should be negative");
    let line_height = asc - desc + gap;
    assert!(line_height > 0.0, "Line height should be positive");

    let m_advance = glyph_advance(&font, 'M').expect("'M' should have an advance");
    assert!(m_advance > 0.0, "'M' advance should be positive");

    let space_advance = glyph_advance(&font, ' ').expect("space should have an advance");
    assert!(space_advance > 0.0, "Space advance should be positive");

    // Sum the per-char advance widths and confirm it's monotonically increasing.
    let test_text = "Hello";
    let mut cursor = 0.0f32;
    let mut last = -1.0f32;
    for ch in test_text.chars() {
        let adv = glyph_advance(&font, ch).unwrap_or(0.0);
        cursor += adv;
        assert!(cursor > last, "cursor should increase per char");
        last = cursor;
    }
    assert!(cursor > 0.0, "Text width should be positive");
}

#[test]
fn test_glyph_id_missing_char() {
    let bytes = font_bytes();
    let font = parse_font(&bytes).expect("parse_font");

    // Emoji is unlikely to be in a monospace font; the lookup should return
    // None rather than panicking.
    let result = glyph_id(&font, '😀');
    let _ = result;
}

#[test]
fn test_empty_text_width() {
    let bytes = font_bytes();
    let font = parse_font(&bytes).expect("parse_font");

    let width: f32 = ""
        .chars()
        .map(|ch| glyph_advance(&font, ch).unwrap_or(0.0))
        .sum();
    assert_eq!(width, 0.0, "Empty text should have 0 width");
}
