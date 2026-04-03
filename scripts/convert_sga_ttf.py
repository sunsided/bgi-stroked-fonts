#!/usr/bin/env python3
"""
Convert Standard Galactic Alphabet TTF font to BGI stroked font format.

The SGA TTF font has filled quadratic bezier outlines. Since BGI uses stroked
line segments, we convert the outlines by:
  1. Properly evaluating quadratic bezier curves into sampled line segments
  2. Scaling to BGI coordinate range
  3. Flipping Y axis (TTF is Y-up, BGI is Y-down)
  4. Simplifying small rounded corners into sharp angular geometry

Usage:
    python3 scripts/convert_sga_ttf.py SgaSmoothRegular-DO0Y3.ttf > src/sga.rs
"""

import sys
import math
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import RecordingPen

# Target height for BGI glyphs (similar to litt.rs which uses ~9-11 units)
TARGET_HEIGHT = 10
# Minimum distance to collapse points (eliminates tiny rounded corner segments)
# At scale 10, the rounded corners are ~0.5 units, so 0.8 collapses them
COLLAPSE_THRESHOLD = 0.8
# Collinear threshold in degrees for merging line segments
COLLINEAR_THRESHOLD_DEG = 10
# Number of samples per quadratic bezier curve segment
BEZIER_SAMPLES = 8


def load_font(ttf_path):
    """Load TTF font and return font object."""
    return TTFont(ttf_path)


def eval_quadratic_bezier(p0, p1, p2, num_samples):
    """Evaluate a quadratic bezier curve and return sampled points.

    p0: start point (on-curve)
    p1: control point (off-curve)
    p2: end point (on-curve)

    The curve is: B(t) = (1-t)^2 * p0 + 2*(1-t)*t * p1 + t^2 * p2
    """
    points = []
    for i in range(1, num_samples + 1):
        t = i / num_samples
        t2 = t * t
        mt = 1 - t
        mt2 = mt * mt
        x = mt2 * p0[0] + 2 * mt * t * p1[0] + t2 * p2[0]
        y = mt2 * p0[1] + 2 * mt * t * p1[1] + t2 * p2[1]
        points.append((x, y))
    return points


def get_glyph_outlines(font, glyph_name):
    """Extract glyph outlines as a list of contours.

    Properly evaluates quadratic bezier curves into sampled line segments.
    Each contour is a list of on-curve points.
    """
    glyph_set = font.getGlyphSet()
    if glyph_name not in glyph_set:
        return []

    pen = RecordingPen()
    glyph_set[glyph_name].draw(pen)

    contours = []
    current_contour = []

    for op, args in pen.value:
        if op == "moveTo":
            if current_contour:
                contours.append(current_contour)
            current_contour = [args[0]]
        elif op == "lineTo":
            current_contour.append(args[0])
        elif op == "qCurveTo":
            # Quadratic bezier: args are [ctrl1, ctrl2, ..., on_curve_end]
            # For a standard qCurveTo with one control point: args = [ctrl, end]
            # For TrueType with multiple off-curve points, intermediate on-curve
            # points are implied at midpoints between consecutive off-curve points.
            if len(args) == 2:
                # Simple case: one control point and one on-curve end point
                p0 = current_contour[-1]
                ctrl = args[0]
                end = args[1]
                sampled = eval_quadratic_bezier(p0, ctrl, end, BEZIER_SAMPLES)
                current_contour.extend(sampled)
            else:
                # Multiple control points with implied on-curve points
                p0 = current_contour[-1]
                controls = list(args[:-1])
                end = args[-1]

                for i, ctrl in enumerate(controls):
                    if i < len(controls) - 1:
                        # Implied on-curve point at midpoint between consecutive controls
                        next_ctrl = controls[i + 1]
                        implied_on = (
                            (ctrl[0] + next_ctrl[0]) / 2,
                            (ctrl[1] + next_ctrl[1]) / 2,
                        )
                        sampled = eval_quadratic_bezier(
                            p0, ctrl, implied_on, BEZIER_SAMPLES
                        )
                        current_contour.extend(sampled)
                        p0 = implied_on
                    else:
                        # Last control point to the actual end point
                        sampled = eval_quadratic_bezier(p0, ctrl, end, BEZIER_SAMPLES)
                        current_contour.extend(sampled)
        elif op == "curveTo":
            # Cubic bezier - sample similarly
            # args = [ctrl1, ctrl2, end]
            if len(args) == 3:
                p0 = current_contour[-1]
                c1, c2, end = args
                for i in range(1, BEZIER_SAMPLES + 1):
                    t = i / BEZIER_SAMPLES
                    mt = 1 - t
                    x = (
                        mt**3 * p0[0]
                        + 3 * mt**2 * t * c1[0]
                        + 3 * mt * t**2 * c2[0]
                        + t**3 * end[0]
                    )
                    y = (
                        mt**3 * p0[1]
                        + 3 * mt**2 * t * c1[1]
                        + 3 * mt * t**2 * c2[1]
                        + t**3 * end[1]
                    )
                    current_contour.append((x, y))
        elif op == "closePath":
            if current_contour:
                contours.append(current_contour)
            current_contour = []
        elif op == "endPath":
            if current_contour:
                contours.append(current_contour)
            current_contour = []

    if current_contour:
        contours.append(current_contour)

    return contours


def get_glyph_bounds(contours):
    """Get bounding box of all contours."""
    if not contours:
        return None

    all_points = [pt for contour in contours for pt in contour]
    if not all_points:
        return None

    min_x = min(pt[0] for pt in all_points)
    max_x = max(pt[0] for pt in all_points)
    min_y = min(pt[1] for pt in all_points)
    max_y = max(pt[1] for pt in all_points)

    return (min_x, min_y, max_x, max_y)


def scale_and_flip_contours(contours, scale, offset_x, max_y):
    """Scale contours and flip Y axis (TTF Y-up -> BGI Y-down).

    After this transform:
    - x' = (x - min_x) * scale  (normalize to origin and scale)
    - y' = (max_y - y) * scale  (flip Y and scale)
    """
    result = []
    for contour in contours:
        new_contour = []
        for pt in contour:
            x = (pt[0] + offset_x) * scale
            y = (max_y - pt[1]) * scale
            new_contour.append((x, y))
        result.append(new_contour)
    return result


def distance(p1, p2):
    """Euclidean distance between two points."""
    return math.sqrt((p1[0] - p2[0]) ** 2 + (p1[1] - p2[1]) ** 2)


def collapse_nearby_points(contour, threshold):
    """Collapse points that are very close together."""
    if len(contour) < 2:
        return contour

    result = [contour[0]]
    for pt in contour[1:]:
        if distance(result[-1], pt) >= threshold:
            result.append(pt)

    # Also check if last point is too close to first
    if len(result) > 1 and distance(result[-1], result[0]) < threshold:
        result = result[:-1]

    return result


def angle_between(p1, p2, p3):
    """Calculate angle at p2 formed by segments p1-p2 and p2-p3."""
    v1 = (p1[0] - p2[0], p1[1] - p2[1])
    v2 = (p3[0] - p2[0], p3[1] - p2[1])

    dot = v1[0] * v2[0] + v1[1] * v2[1]
    len1 = math.sqrt(v1[0] ** 2 + v1[1] ** 2)
    len2 = math.sqrt(v2[0] ** 2 + v2[1] ** 2)

    if len1 < 0.001 or len2 < 0.001:
        return 180  # Degenerate case

    cos_angle = max(-1, min(1, dot / (len1 * len2)))
    return math.degrees(math.acos(cos_angle))


def merge_collinear_segments(contour, threshold_deg):
    """Merge consecutive segments that are nearly collinear."""
    if len(contour) < 3:
        return contour

    result = [contour[0]]

    for i in range(1, len(contour)):
        p1 = result[-1]
        p2 = contour[i]

        # Look ahead to next point (wrap around for closed contours)
        p3_idx = (i + 1) % len(contour)
        p3 = contour[p3_idx]

        # Check if p1-p2-p3 is nearly straight
        angle = angle_between(p1, p2, p3)

        # If angle is close to 180 (straight line), skip p2
        if abs(180 - angle) < threshold_deg:
            continue

        result.append(p2)

    return result


def simplify_contour(contour, collapse_threshold, collinear_threshold_deg):
    """Simplify a contour by collapsing points and merging collinear segments."""
    if len(contour) < 2:
        return contour

    # First pass: collapse nearby points
    simplified = collapse_nearby_points(contour, collapse_threshold)

    # Second pass: merge collinear segments
    simplified = merge_collinear_segments(simplified, collinear_threshold_deg)

    return simplified


def contours_to_segments(contours):
    """Convert contours to line segments for BGI format."""
    segments = []
    for contour in contours:
        if len(contour) < 2:
            continue
        # Create segments between consecutive points
        for i in range(len(contour)):
            p1 = contour[i]
            p2 = contour[(i + 1) % len(contour)]
            # Round to integers for BGI format
            x0, y0 = round(p1[0]), round(p1[1])
            x1, y1 = round(p2[0]), round(p2[1])
            # Skip degenerate segments
            if x0 == x1 and y0 == y1:
                continue
            segments.append((x0, y0, x1, y1))
    return segments


def get_cmap(font):
    """Get unicode to glyph name mapping (keys are integer code points)."""
    cmap = {}
    for table in font["cmap"].tables:
        if hasattr(table, "cmap"):
            cmap.update(table.cmap)
    return cmap


def process_glyph(font, glyph_name, target_height):
    """Process a single glyph and return BGI segments."""
    contours = get_glyph_outlines(font, glyph_name)
    if not contours:
        return [], 0

    bounds = get_glyph_bounds(contours)
    if not bounds:
        return [], 0

    min_x, min_y, max_x, max_y = bounds
    width = max_x - min_x
    height = max_y - min_y

    if height < 1:
        return [], 0

    # Scale to target height
    scale = target_height / height

    # Scale, normalize to origin, and flip Y axis
    scaled = scale_and_flip_contours(contours, scale, -min_x, max_y)

    # Simplify each contour
    simplified = [
        simplify_contour(c, COLLAPSE_THRESHOLD, COLLINEAR_THRESHOLD_DEG) for c in scaled
    ]

    # Filter empty contours
    simplified = [c for c in simplified if len(c) >= 2]

    # Convert to segments
    segments = contours_to_segments(simplified)

    # Calculate width (in scaled units)
    glyph_width = round(width * scale)

    return segments, glyph_width


def generate_rust_glyph(name_upper, glyph_num, segments, width):
    """Generate Rust code for a single glyph."""
    lines = []

    # Width constant
    lines.append(f"/// Width (pixels) for [`{name_upper}_{glyph_num}`] glyph data.")
    lines.append(f'#[doc(alias = "{name_upper.lower()}_{glyph_num}_width")]')
    lines.append(f"pub const {name_upper}_{glyph_num}_WIDTH: u8 = {width};")
    lines.append("")

    # Glyph data
    lines.append(
        f"/// Glyph data; see [`{name_upper}_{glyph_num}_WIDTH`] and [`{name_upper}_{glyph_num}_SIZE`]."
    )
    lines.append(f'#[doc(alias = "{name_upper.lower()}_{glyph_num}")]')
    lines.append("#[rustfmt::skip]")

    size = len(segments) * 4
    lines.append(
        f"pub const {name_upper}_{glyph_num}: [i8; {name_upper}_{glyph_num}_SIZE as usize] = ["
    )

    for seg in segments:
        x0, y0, x1, y1 = seg
        lines.append(f"    {x0:3}, {y0:3}, {x1:3}, {y1:3},")

    lines.append("];")
    lines.append("")

    # Size constant
    lines.append(
        f"/// Byte count for [`{name_upper}_{glyph_num}`] (number of entries)."
    )
    lines.append(f'#[doc(alias = "{name_upper.lower()}_{glyph_num}_size")]')
    lines.append(f"pub const {name_upper}_{glyph_num}_SIZE: u16 = {size};")
    lines.append("")

    return lines


def generate_rust_file(font, ttf_path):
    """Generate complete Rust file for SGA font."""
    cmap = get_cmap(font)

    lines = []
    lines.append("//! Standard Galactic Alphabet (SGA) stroked font.")
    lines.append("//!")
    lines.append("//! Converted from TTF using scripts/convert_sga_ttf.py")
    lines.append(
        "//! The Standard Galactic Alphabet is a cipher used in Commander Keen,"
    )
    lines.append("//! Minecraft, and other games.")
    lines.append("")

    name_upper = "SGA"

    # Process glyphs for ASCII range 32-126 (standard printable ASCII)
    # The BGI format uses 1-indexed glyph numbers starting from space (index 1 = space)
    # Glyph 1 = space (ASCII 32), Glyph 2 = ! (ASCII 33), etc.

    glyph_data = {}  # glyph_num -> (segments, width)

    # SGA only has A-Z glyphs (same for upper and lower case)
    # Map ASCII letters to SGA glyphs
    # NOTE: cmap uses integer code points as keys, not character strings
    for ascii_code in range(32, 127):  # ASCII 32-126
        glyph_num = ascii_code - 31  # Glyph 1 = ASCII 32

        if ascii_code in cmap:
            glyph_name = cmap[ascii_code]
            segments, width = process_glyph(font, glyph_name, TARGET_HEIGHT)
            glyph_data[glyph_num] = (segments, width if width > 0 else 6)
        elif ord(chr(ascii_code).upper()) in cmap:
            # Try uppercase version (for lowercase letters mapping to same SGA glyph)
            glyph_name = cmap[ord(chr(ascii_code).upper())]
            segments, width = process_glyph(font, glyph_name, TARGET_HEIGHT)
            glyph_data[glyph_num] = (segments, width if width > 0 else 6)
        else:
            # Empty glyph
            glyph_data[glyph_num] = ([], 6)

    # Generate individual glyph constants
    for glyph_num in sorted(glyph_data.keys()):
        segments, width = glyph_data[glyph_num]
        glyph_lines = generate_rust_glyph(name_upper, glyph_num, segments, width)
        lines.extend(glyph_lines)

    # Generate aggregate constants
    num_glyphs = len(glyph_data)

    # HEIGHT constant (based on target)
    lines.append(f'#[doc(alias = "{name_upper.lower()}_height")]')
    lines.append(f"pub const {name_upper}_HEIGHT: i8 = {TARGET_HEIGHT};")
    lines.append("")

    # DESC_HEIGHT constant (no descenders for SGA)
    lines.append(f'#[doc(alias = "{name_upper.lower()}_desc_height")]')
    lines.append(f"pub const {name_upper}_DESC_HEIGHT: i8 = 0;")
    lines.append("")

    # WIDTH array
    lines.append(f"/// Glyph widths. Length inferred.")
    lines.append(f'#[doc(alias = "{name_upper.lower()}_width")]')
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub const {name_upper}_WIDTH: [u8; {num_glyphs}] = [")

    width_items = []
    for glyph_num in sorted(glyph_data.keys()):
        width_items.append(f"{name_upper}_{glyph_num}_WIDTH")

    # Format in rows of 4
    for i in range(0, len(width_items), 4):
        chunk = width_items[i : i + 4]
        lines.append("    " + ", ".join(chunk) + ",")

    lines.append("];")
    lines.append("")

    # SIZE array
    lines.append(f'#[doc(alias = "{name_upper.lower()}_size")]')
    lines.append(f"pub const {name_upper}_SIZE: [u16; {num_glyphs}] = [")

    for glyph_num in sorted(glyph_data.keys()):
        lines.append(f"    {name_upper}_{glyph_num}_SIZE,")

    lines.append("];")
    lines.append("")

    # Main array of glyph references
    lines.append(f'#[doc(alias = "{name_upper.lower()}")]')
    lines.append(f"pub const {name_upper}: [&[i8]; {num_glyphs}] = [")

    glyph_refs = []
    for glyph_num in sorted(glyph_data.keys()):
        glyph_refs.append(f"&{name_upper}_{glyph_num}")

    # Format in rows of 10
    for i in range(0, len(glyph_refs), 10):
        chunk = glyph_refs[i : i + 10]
        lines.append("    " + ", ".join(chunk) + ",")

    lines.append("];")

    return "\n".join(lines)


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 convert_sga_ttf.py <ttf_path>", file=sys.stderr)
        print("Output is written to stdout", file=sys.stderr)
        sys.exit(1)

    ttf_path = sys.argv[1]
    font = load_font(ttf_path)

    rust_code = generate_rust_file(font, ttf_path)
    print(rust_code)


if __name__ == "__main__":
    main()
