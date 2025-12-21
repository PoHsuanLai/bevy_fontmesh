//! Tests for font metrics API

use bevy_fontmesh::FontMesh;
use std::fs;

fn load_test_font() -> FontMesh {
    let font_bytes = fs::read("assets/fonts/FiraMono-Medium.ttf")
        .expect("Failed to load test font - make sure assets/fonts/FiraMono-Medium.ttf exists");
    FontMesh { data: font_bytes }
}

#[test]
fn test_font_metrics_api() {
    let font = load_test_font();

    // Test font-level metrics
    let metrics = font.font_metrics().expect("Should get font metrics");
    assert!(metrics.ascender > 0.0, "Ascender should be positive");
    assert!(metrics.descender < 0.0, "Descender should be negative");
    assert!(metrics.line_height > 0.0, "Line height should be positive");
    assert_eq!(
        metrics.line_height,
        metrics.ascender - metrics.descender + metrics.line_gap,
        "Line height should equal ascender - descender + line_gap"
    );

    // Test glyph metrics
    let glyph_m = font.glyph_metrics('M').expect("Should get metrics for 'M'");
    assert!(glyph_m.advance > 0.0, "Advance should be positive");
    assert!(glyph_m.has_outline, "'M' should have outline");

    let glyph_space = font
        .glyph_metrics(' ')
        .expect("Should get metrics for space");
    assert!(
        glyph_space.advance > 0.0,
        "Space advance should be positive"
    );
    assert!(!glyph_space.has_outline, "Space should not have outline");

    // Test text width
    let test_text = "Hello";
    let width = font.text_width(test_text);
    assert!(width > 0.0, "Text width should be positive");

    // Verify width equals sum of advances
    let manual_width: f32 = test_text
        .chars()
        .filter_map(|ch| font.glyph_metrics(ch))
        .map(|m| m.advance)
        .sum();
    assert!(
        (width - manual_width).abs() < 0.001,
        "text_width() should equal sum of advances"
    );

    // Test char positions
    let positions = font.char_positions(test_text);
    assert_eq!(
        positions.len(),
        test_text.len(),
        "Should have position for each character"
    );

    // First character should be at x=0
    assert_eq!(positions[0].0, 0);
    assert_eq!(positions[0].1, 0.0);

    // Positions should be monotonically increasing
    for i in 1..positions.len() {
        assert!(
            positions[i].1 >= positions[i - 1].1,
            "Positions should increase"
        );
    }
}

#[test]
fn test_glyph_metrics_missing_char() {
    let font = load_test_font();

    // Test with an emoji character unlikely to be in a monospace font
    let result = font.glyph_metrics('😀');
    // Note: Some fonts may include fallback glyphs, so we just verify the API works
    // Either Some or None is acceptable - we're testing that it doesn't crash
    let _ = result;
}

#[test]
fn test_empty_text_width() {
    let font = load_test_font();

    assert_eq!(font.text_width(""), 0.0, "Empty text should have 0 width");
    assert_eq!(
        font.char_positions("").len(),
        0,
        "Empty text should have no positions"
    );
}
