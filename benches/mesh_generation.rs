//! Benchmarks for bevy_fontmesh mesh generation
//!
//! These benchmarks measure the core mesh generation pipeline:
//! font parsing, per-character mesh generation, vertex assembly,
//! and justification/anchor offset calculation.
//!
//! Note: in 0.4 the production code shapes via cosmic-text rather than the
//! hand-rolled advance-based layout this file mirrors. The bench keeps the
//! simple layout (cursor_x += advance) because it isolates the parts of the
//! pipeline that *we* still own — outline tessellation and mesh assembly.
//! See `update_text_meshes` in `src/system.rs` for the real layout path.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fontmesh::{
    ascender, descender, glyph_advance, glyph_id, glyph_to_mesh_3d, line_gap, parse_font, FontRef,
    GlyphId,
};

const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/FiraMono-Medium.ttf");

// ── Helpers ─────────────────────────────────────────────────────────────────

fn advance(ch: char, font: &FontRef) -> f32 {
    glyph_advance(font, ch).unwrap_or_else(|| {
        if ch.is_whitespace() {
            (ascender(font) - descender(font)) * 0.25
        } else {
            0.0
        }
    })
}

fn line_width(line: &str, font: &FontRef) -> f32 {
    line.chars().map(|ch| advance(ch, font)).sum()
}

/// Reproduce a center-justified version of the old text layout, without ECS.
fn generate_text_mesh(
    text: &str,
    depth: f32,
    subdivision: u8,
    font: &FontRef,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    let mut all_normals: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();

    let lh = ascender(font) - descender(font) + line_gap(font);
    let mut cursor_y = 0.0f32;
    let mut index_offset = 0u32;

    for line in text.split('\n') {
        let mut cursor_x = -line_width(line, font) * 0.5; // center
        for ch in line.chars() {
            if ch.is_whitespace() {
                cursor_x += advance(ch, font);
                continue;
            }
            let Some(gid) = glyph_id(font, ch) else {
                continue;
            };
            if let Ok(mesh) = glyph_to_mesh_3d(font, GlyphId::new(gid.to_u32()), depth, subdivision)
            {
                all_vertices.extend(
                    mesh.vertices
                        .iter()
                        .map(|v| [v.x + cursor_x, v.y + cursor_y, v.z]),
                );
                all_normals.extend(mesh.normals.iter().map(|n| [n.x, n.y, n.z]));
                all_indices.extend(mesh.indices.iter().map(|i| i + index_offset));
                index_offset += mesh.vertices.len() as u32;
                cursor_x += advance(ch, font);
            }
        }
        cursor_y -= lh;
    }

    (all_vertices, all_normals, all_indices)
}

// ── Benchmarks ─────────────────────────────────────────────────────────────

fn bench_font_parsing(c: &mut Criterion) {
    c.bench_function("font_parse", |b| {
        b.iter(|| parse_font(black_box(FONT_DATA)).unwrap())
    });
}

fn bench_single_char(c: &mut Criterion) {
    let font = parse_font(FONT_DATA).unwrap();
    let gid = glyph_id(&font, 'A').unwrap();
    let mut group = c.benchmark_group("single_char");
    for subdivisions in [5u8, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(subdivisions),
            &subdivisions,
            |b, &sub| {
                b.iter(|| glyph_to_mesh_3d(black_box(&font), black_box(gid), 0.5, sub).unwrap())
            },
        );
    }
    group.finish();
}

fn bench_word(c: &mut Criterion) {
    let font = parse_font(FONT_DATA).unwrap();
    let mut group = c.benchmark_group("word");
    for word in ["Hi", "Hello", "Hello World", "The quick brown fox"] {
        group.bench_with_input(BenchmarkId::from_parameter(word), word, |b, text| {
            b.iter(|| generate_text_mesh(black_box(text), 0.5, 20, black_box(&font)))
        });
    }
    group.finish();
}

fn bench_multiline(c: &mut Criterion) {
    let font = parse_font(FONT_DATA).unwrap();
    let texts = [
        ("2_lines", "Hello\nWorld"),
        ("4_lines", "Line one\nLine two\nLine three\nLine four"),
        (
            "8_lines",
            "Line one\nLine two\nLine three\nLine four\nLine five\nLine six\nLine seven\nLine eight",
        ),
    ];
    let mut group = c.benchmark_group("multiline");
    for (name, text) in &texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), text, |b, text| {
            b.iter(|| generate_text_mesh(black_box(text), 0.5, 20, black_box(&font)))
        });
    }
    group.finish();
}

fn bench_quality_levels(c: &mut Criterion) {
    let font = parse_font(FONT_DATA).unwrap();
    let mut group = c.benchmark_group("quality");
    for subdivisions in [5u8, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(subdivisions),
            &subdivisions,
            |b, &sub| b.iter(|| generate_text_mesh(black_box("Hello"), 0.5, sub, black_box(&font))),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_font_parsing,
    bench_single_char,
    bench_word,
    bench_multiline,
    bench_quality_levels,
);
criterion_main!(benches);
