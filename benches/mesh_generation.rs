//! Benchmarks for bevy_fontmesh mesh generation
//!
//! These benchmarks measure the core mesh generation pipeline:
//! font parsing, per-character mesh generation, vertex assembly,
//! and justification/anchor offset calculation.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/FiraMono-Medium.ttf");

// ── Helpers mirroring system.rs logic ────────────────────────────────────────

fn get_glyph_advance(ch: char, face: &fontmesh::Face) -> f32 {
    fontmesh::glyph_advance(face, ch).unwrap_or_else(|| {
        if ch.is_whitespace() {
            (fontmesh::ascender(face) - fontmesh::descender(face)) * 0.25
        } else {
            0.0
        }
    })
}

fn calculate_line_width(line: &str, face: &fontmesh::Face) -> f32 {
    line.chars().map(|ch| get_glyph_advance(ch, face)).sum()
}

/// Reproduce the core of `update_text_meshes` without Bevy ECS.
fn generate_text_mesh(
    text: &str,
    depth: f32,
    subdivision: u8,
    face: &fontmesh::Face,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    let mut all_normals: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();

    let line_height =
        fontmesh::ascender(face) - fontmesh::descender(face) + fontmesh::line_gap(face);

    let mut cursor_y = 0.0f32;
    let mut index_offset = 0u32;

    for line in text.split('\n') {
        let line_width = calculate_line_width(line, face);
        let mut cursor_x = -line_width * 0.5; // center justification

        for ch in line.chars() {
            if ch.is_whitespace() {
                cursor_x += get_glyph_advance(ch, face);
                continue;
            }

            if let Ok(mesh) = fontmesh::char_to_mesh_3d(face, ch, depth, subdivision) {
                all_vertices.extend(
                    mesh.vertices
                        .iter()
                        .map(|v| [v.x + cursor_x, v.y + cursor_y, v.z]),
                );
                all_normals.extend(mesh.normals.iter().map(|n| [n.x, n.y, n.z]));
                all_indices.extend(mesh.indices.iter().map(|i| i + index_offset));
                index_offset += mesh.vertices.len() as u32;
                cursor_x += get_glyph_advance(ch, face);
            }
        }

        cursor_y -= line_height;
    }

    (all_vertices, all_normals, all_indices)
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

/// Baseline: cost of parsing the font face from bytes each time.
fn bench_font_parsing(c: &mut Criterion) {
    c.bench_function("font_parse", |b| {
        b.iter(|| fontmesh::Face::parse(black_box(FONT_DATA), 0).unwrap())
    });
}

/// Single character mesh generation at different quality levels.
fn bench_single_char(c: &mut Criterion) {
    let face = fontmesh::Face::parse(FONT_DATA, 0).unwrap();
    let mut group = c.benchmark_group("single_char");

    for subdivisions in [5u8, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(subdivisions),
            &subdivisions,
            |b, &sub| {
                b.iter(|| {
                    fontmesh::char_to_mesh_3d(black_box(&face), black_box('A'), 0.5, sub).unwrap()
                })
            },
        );
    }
    group.finish();
}

/// Full word mesh generation (vertex assembly + justification).
fn bench_word(c: &mut Criterion) {
    let face = fontmesh::Face::parse(FONT_DATA, 0).unwrap();
    let mut group = c.benchmark_group("word");

    for word in ["Hi", "Hello", "Hello World", "The quick brown fox"] {
        group.bench_with_input(BenchmarkId::from_parameter(word), word, |b, text| {
            b.iter(|| generate_text_mesh(black_box(text), 0.5, 20, black_box(&face)))
        });
    }
    group.finish();
}

/// Multiline text generation.
fn bench_multiline(c: &mut Criterion) {
    let face = fontmesh::Face::parse(FONT_DATA, 0).unwrap();

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
            b.iter(|| generate_text_mesh(black_box(text), 0.5, 20, black_box(&face)))
        });
    }
    group.finish();
}

/// Impact of subdivision quality on a fixed short string.
fn bench_quality_levels(c: &mut Criterion) {
    let face = fontmesh::Face::parse(FONT_DATA, 0).unwrap();
    let mut group = c.benchmark_group("quality");

    for subdivisions in [5u8, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(subdivisions),
            &subdivisions,
            |b, &sub| b.iter(|| generate_text_mesh(black_box("Hello"), 0.5, sub, black_box(&face))),
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
