# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-01

[0.1.0]: https://github.com/sunsided/bgi-stroked-fonts/releases/tag/v0.1.0

### Added

- **Core BGI Font Library** - Pure Rust implementation of Borland Graphics Interface stroked fonts
    - Complete font data conversion from SDL_bgi C headers to Rust modules
    - 10 classic BGI font styles: Bold, Euro, Gothic, Small Complex, Small, Sans-serif, Script, Simple, Triplex, Triplex Script
    - Character width and size information for proper text layout
    - Stroke-based vector font data for scalable rendering

- **Font Data Structure**
    - Font stroke data arrays (`FONT`) containing line segments for each character (0-255)
    - Character width arrays (`FONT_WIDTH`) for horizontal spacing calculations
    - Character size arrays (`FONT_SIZE`) for memory layout information
    - Stroke format: 4-byte segments representing line endpoints [x0, y0, x1, y1]

- **Feature Flag System**
    - Granular feature flags for each font to minimize binary size
    - Default features include all fonts for convenience
    - Individual font features: `bold`, `euro`, `goth`, `lcom`, `litt`, `sans`, `scri`, `simp`, `trip`, `tscr`
    - Optional conversion utilities feature (`_convert`) with regex dependency

- **Font Rendering Examples**
    - Complete showcase example demonstrating all font styles
    - Two-scale grid rendering for font comparison
    - PGM image output for visualization
    - Floating-point scaling with proper glyph normalization
    - Bounding box calculations and metrics computation

- **Documentation and Assets**
    - Font sample images (Gothic, Small, Simple) in `docs/` directory
    - Comprehensive API documentation with usage examples
    - README with installation, usage, and feature flag instructions
    - License information with proper SDL_bgi attribution

### Technical Details

- **Memory Safety**: 100% safe Rust code with `#![forbid(unsafe_code)]`
- **Zero Dependencies**: Core library has no external dependencies (regex only for optional conversion tools)
- **Compatibility**: Maintains original BGI font data format and character encoding
- **Performance**: Static font data with no runtime initialization overhead
- **Cross-platform**: Pure Rust implementation works on all supported platforms
