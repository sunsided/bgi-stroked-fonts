//! Simple 8-bit rasterization of BGI stroked glyphs (SDL_BGI-style) with grid layout + f32 scale.

use bgi_stroked_fonts::{bold, euro, goth, lcom, litt, sans, scri, simp, trip, tscr};
use std::cmp::max;

fn main() {
    // Two showcase scales (tweak as you like)
    let s1: f32 = 0.6;
    let s2: f32 = 1.2;

    // Grid config
    let cols: usize = 24; // how many glyphs per row
    let cell_padding_px: i32 = 4; // inner padding per cell
    let row_gap_px: i32 = 12; // gap between the two grids

    // Bold font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &bold::BOLD_WIDTH,
        &bold::BOLD_SIZE,
        &bold::BOLD,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("bold_demo_grid.pgm"), w, h, &buf).unwrap();

    // Euro font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &euro::EURO_WIDTH,
        &euro::EURO_SIZE,
        &euro::EURO,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("euro_demo_grid.pgm"), w, h, &buf).unwrap();

    // Goth font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &goth::GOTH_WIDTH,
        &goth::GOTH_SIZE,
        &goth::GOTH,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("goth_demo_grid.pgm"), w, h, &buf).unwrap();

    // lcom font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &lcom::LCOM_WIDTH,
        &lcom::LCOM_SIZE,
        &lcom::LCOM,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("lcom_demo_grid.pgm"), w, h, &buf).unwrap();

    // litt font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &litt::LITT_WIDTH,
        &litt::LITT_SIZE,
        &litt::LITT,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("litt_demo_grid.pgm"), w, h, &buf).unwrap();

    // sans font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &sans::SANS_WIDTH,
        &sans::SANS_SIZE,
        &sans::SANS,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("sans_demo_grid.pgm"), w, h, &buf).unwrap();

    // simp font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &simp::SIMP_WIDTH,
        &simp::SIMP_SIZE,
        &simp::SIMP,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("simp_demo_grid.pgm"), w, h, &buf).unwrap();

    // scri font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &scri::SCRI_WIDTH,
        &scri::SCRI_SIZE,
        &scri::SCRI,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("scri_demo_grid.pgm"), w, h, &buf).unwrap();

    // trip font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &trip::TRIP_WIDTH,
        &trip::TRIP_SIZE,
        &trip::TRIP,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("trip_demo_grid.pgm"), w, h, &buf).unwrap();

    // tscr font demo
    let (w, h, mut buf) = render_showcase_two_grids(
        cols,
        cell_padding_px,
        row_gap_px,
        s1,
        s2,
        &tscr::TSCR_WIDTH,
        &tscr::TSCR_SIZE,
        &tscr::TSCR,
    );
    invert_in_place(&mut buf);
    write_pgm(std::path::Path::new("tscr_demo_grid.pgm"), w, h, &buf).unwrap();
}

/// Write the buffer as a binary PGM file (no external deps).
pub fn write_pgm(path: &std::path::Path, w: usize, h: usize, buf: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    writeln!(f, "P5\n{} {}\n255", w, h)?;
    f.write_all(buf)?;
    Ok(())
}

/// Flips white-on-black pixels to black-on-white.
pub fn invert_in_place(buf: &mut [u8]) {
    for px in buf.iter_mut() {
        *px = 255u8.wrapping_sub(*px);
    }
}

/// Compute glyph bbox (min_x, min_y, max_x, max_y) from BGI stroked data.
/// Returns None for empty glyphs.
pub fn glyph_bbox(bytes: &[i8]) -> Option<(i32, i32, i32, i32)> {
    let mut it = bytes.chunks_exact(4);
    let mut first = true;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0, 0, 0, 0);

    for seg in it.by_ref() {
        let x0 = seg[0] as i32;
        let y0 = seg[1] as i32;
        let x1 = seg[2] as i32;
        let y1 = seg[3] as i32;
        if first {
            min_x = x0.min(x1);
            max_x = x0.max(x1);
            min_y = y0.min(y1);
            max_y = y0.max(y1);
            first = false;
        } else {
            min_x = min_x.min(x0.min(x1));
            max_x = max_x.max(x0.max(x1));
            min_y = min_y.min(y0.min(y1));
            max_y = max_y.max(y0.max(y1));
        }
    }
    if first {
        None
    } else {
        Some((min_x, min_y, max_x, max_y))
    }
}

#[inline]
pub fn set_px(buf: &mut [u8], w: usize, h: usize, x: i32, y: i32) {
    if x >= 0 && y >= 0 {
        let (x, y) = (x as usize, y as usize);
        if x < w && y < h {
            buf[y * w + x] = 255;
        }
    }
}

/// Integer Bresenham
pub fn draw_line(buf: &mut [u8], w: usize, h: usize, mut x0: i32, mut y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        set_px(buf, w, h, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Render a single glyph at floating-point scale into an origin.
/// Normalizes both left/top by subtracting (min_x, min_y) so the glyph fits the cell.
pub fn render_glyph_into_scaled(
    buf: &mut [u8],
    w: usize,
    h: usize,
    origin_x: i32,
    origin_y: i32,
    glyph_bytes: &[i8],
    scale: f32,
    normalize_left_and_top: bool,
) {
    let (nx, ny) = if normalize_left_and_top {
        glyph_bbox(glyph_bytes)
            .map(|(min_x, min_y, _, _)| (min_x, min_y))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    for seg in glyph_bytes.chunks_exact(4) {
        let x0 = ((seg[0] as i32 - nx) as f32 * scale).round() as i32 + origin_x;
        let y0 = ((seg[1] as i32 - ny) as f32 * scale).round() as i32 + origin_y;
        let x1 = ((seg[2] as i32 - nx) as f32 * scale).round() as i32 + origin_x;
        let y1 = ((seg[3] as i32 - ny) as f32 * scale).round() as i32 + origin_y;
        draw_line(buf, w, h, x0, y0, x1, y1);
    }
}

/// Decide the horizontal advance in *font units* (robust).
/// Prefer `width_units`; fall back to bbox width if width is 0/suspicious.
pub fn glyph_advance_units(bytes: &[i8], width_units: u8) -> i32 {
    let min_advance = 1;
    let width_units = width_units as i32;
    let bbox_units = glyph_bbox(bytes)
        .map(|(min_x, _min_y, max_x, _max_y)| (max_x - min_x + 1).max(min_advance))
        .unwrap_or(min_advance);
    let advance = if width_units <= 0 {
        bbox_units
    } else {
        width_units
    };
    advance.max(min_advance)
}

/// Compute max bbox width/height across the font (in font units).
pub fn font_metrics(glyph_widths: &[u8], glyph_data: &[&'static [i8]]) -> (i32, i32) {
    let mut max_w = 1;
    let mut max_h = 1;
    for (i, bytes) in glyph_data.iter().enumerate() {
        if bytes.is_empty() {
            continue;
        }
        let (w_units, h_units) = if let Some((min_x, min_y, max_x, max_y)) = glyph_bbox(bytes) {
            (max_x - min_x + 1, max_y - min_y + 1)
        } else {
            (1, 1)
        };
        let adv_units = glyph_advance_units(bytes, *glyph_widths.get(i).unwrap_or(&0));
        max_w = max(max_w, max(adv_units, w_units));
        max_h = max(max_h, h_units);
    }
    (max_w, max_h)
}

/// Render a *grid* of glyphs at a given scale. Returns canvas size and buffer.
///
/// - `cols`: number of glyphs per row
/// - `cell_padding_px`: padding around each cell (applied on all sides)
/// - `y_offset`: top offset where this grid should be drawn (to stack multiple grids)
pub fn render_glyph_grid(
    cols: usize,
    cell_padding_px: i32,
    y_offset: i32,
    scale: f32,
    glyph_widths: &[u8],
    glyph_sizes: &[u16],
    glyph_data: &[&'static [i8]],
    canvas: &mut [u8],
    canvas_w: usize,
    canvas_h: usize,
) {
    // Compute per-cell size in pixels from font metrics
    let (max_w_units, max_h_units) = font_metrics(glyph_widths, glyph_data);
    let cell_w_px = (max_w_units as f32 * scale).ceil() as i32 + cell_padding_px * 2;
    let cell_h_px = (max_h_units as f32 * scale).ceil() as i32 + cell_padding_px * 2;

    for (idx, bytes) in glyph_data.iter().enumerate() {
        let row = (idx / cols) as i32;
        let col = (idx % cols) as i32;

        let ox = col * cell_w_px + cell_padding_px;
        let oy = y_offset + row * cell_h_px + cell_padding_px;

        // Sanity: sizes are informational only
        let _declared = glyph_sizes.get(idx).copied().unwrap_or(bytes.len() as u16);
        let _ = _declared; // keep quiet

        render_glyph_into_scaled(
            canvas, canvas_w, canvas_h, ox, oy, bytes, scale,
            true, // normalize both left & top so strokes fit cell
        );
    }
}

/// Prepare a canvas and draw two grids: one at `s1`, then below it one at `s2`.
/// Returns (w, h, buffer).
pub fn render_showcase_two_grids(
    cols: usize,
    cell_padding_px: i32,
    row_gap_px: i32,
    s1: f32,
    s2: f32,
    glyph_widths: &[u8],
    glyph_sizes: &[u16],
    glyph_data: &[&'static [i8]],
) -> (usize, usize, Vec<u8>) {
    let n = glyph_data.len();

    // Cell sizes for each scale
    let (max_w_units, max_h_units) = font_metrics(glyph_widths, glyph_data);
    let cell_w_1 = (max_w_units as f32 * s1).ceil() as i32 + cell_padding_px * 2;
    let cell_h_1 = (max_h_units as f32 * s1).ceil() as i32 + cell_padding_px * 2;

    let cell_w_2 = (max_w_units as f32 * s2).ceil() as i32 + cell_padding_px * 2;
    let cell_h_2 = (max_h_units as f32 * s2).ceil() as i32 + cell_padding_px * 2;

    let rows = ((n + cols - 1) / cols) as i32;

    // We use the max width of the two grids for a common canvas width.
    let w_px = (cols as i32 * max(cell_w_1, cell_w_2)) as usize;
    let h_px = (rows * cell_h_1 + row_gap_px + rows * cell_h_2) as usize;

    let mut buf = vec![0u8; w_px * h_px];

    // First grid at top
    render_glyph_grid(
        cols,
        cell_padding_px,
        0, // y_offset
        s1,
        glyph_widths,
        glyph_sizes,
        glyph_data,
        &mut buf,
        w_px,
        h_px,
    );

    // Second grid stacked below, with a gap
    let y_offset_2 = rows * cell_h_1 + row_gap_px;
    render_glyph_grid(
        cols,
        cell_padding_px,
        y_offset_2,
        s2,
        glyph_widths,
        glyph_sizes,
        glyph_data,
        &mut buf,
        w_px,
        h_px,
    );

    (w_px, h_px, buf)
}
