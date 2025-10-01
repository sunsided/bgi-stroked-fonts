// src/main.rs
// Convert SDL_BGI-style C headers into Rust constants with doc aliases,
// keeping glyph width/array/size triplets together and cross-linked.

// Usage: cargo run -- <dir-with-h-files>

use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Define {
    name: String,
    value: String,
    idx: usize, // first-seen order
}

#[derive(Debug, Clone)]
struct CharArray {
    name: String,
    body_raw: String,
    idx: usize,
}

#[derive(Debug, Clone)]
struct IntArray {
    name: String,
    values: Vec<String>,
    idx: usize,
}

#[derive(Debug, Clone)]
struct PointerArray {
    name: String,
    targets: Vec<String>,
    idx: usize,
}

#[derive(Debug, Default)]
struct FileIr {
    header_comments: Vec<String>,
    defines: Vec<Define>,
    char_arrays: Vec<CharArray>,
    int_arrays: Vec<IntArray>,
    pointer_arrays: Vec<PointerArray>,
}

#[derive(Debug, Default)]
struct Glyph {
    base: String,             // e.g., "bold_2"
    width: Option<Define>,    // name "bold_2_width"
    array: Option<CharArray>, // name "bold_2"
    size: Option<Define>,     // name "bold_2_size"
    order_key: usize,         // min idx among parts
}

fn upper_snake(s: &str) -> String {
    s.to_ascii_uppercase()
}

fn collect_block(lines: &[String], mut i: usize) -> (String, usize) {
    // Collect from the line that contains '{' through the line that contains "};"
    let mut buf = String::new();
    let mut opened = false;
    while i < lines.len() {
        let line = &lines[i];
        if !opened {
            if let Some(pos) = line.find('{') {
                opened = true;
                let after = &line[pos + 1..];
                if after.contains("};") {
                    let before_end = after.splitn(2, "};").next().unwrap_or("").to_string();
                    return (before_end, i);
                }
                if !after.trim().is_empty() {
                    buf.push_str(after);
                    if !buf.ends_with('\n') {
                        buf.push('\n');
                    }
                }
            }
        } else {
            if let Some(pos) = line.find("};") {
                let part = &line[..pos];
                buf.push_str(part);
                return (buf, i);
            } else {
                buf.push_str(line);
                if !buf.ends_with('\n') {
                    buf.push('\n');
                }
            }
        }
        i += 1;
    }
    (buf, i)
}

fn parse_h(text: &str) -> FileIr {
    let mut ir = FileIr::default();

    let re_define = Regex::new(r#"^\s*#define\s+([A-Za-z0-9_]+)\s+([^\s/][^\r\n]*)"#).unwrap();
    let re_static_char =
        Regex::new(r#"^\s*static\s+(?:const\s+)?char\s+([A-Za-z0-9_]+)\s*\[\s*\]\s*=\s*\{"#)
            .unwrap();
    let re_static_int_array =
        Regex::new(r#"^\s*static\s+const\s+int\s+([A-Za-z0-9_]+)\s*\[\s*\]\s*=\s*\{"#).unwrap();
    let re_static_char_ptr_array =
        Regex::new(r#"^\s*static\s+const\s+char\s*\*\s*([A-Za-z0-9_]+)\s*\[\s*\]\s*=\s*\{"#)
            .unwrap();

    let lines: Vec<String> = text.lines().map(|s| format!("{}\n", s)).collect();

    let mut i = 0usize;
    let mut seen_counter = 0usize;
    while i < lines.len() {
        let line = &lines[i];

        if line.trim_start().starts_with("//") {
            if ir.defines.is_empty()
                && ir.char_arrays.is_empty()
                && ir.int_arrays.is_empty()
                && ir.pointer_arrays.is_empty()
            {
                ir.header_comments.push(line.trim_end().to_string());
            }
            i += 1;
            continue;
        }

        if let Some(m) = re_define.captures(line) {
            let name = m.get(1).unwrap().as_str().trim().to_string();
            let val = m
                .get(2)
                .unwrap()
                .as_str()
                .trim()
                .trim_end_matches('\n')
                .to_string();
            ir.defines.push(Define {
                name,
                value: val,
                idx: seen_counter,
            });
            seen_counter += 1;
            i += 1;
            continue;
        }

        if let Some(m) = re_static_char.captures(line) {
            let name = m.get(1).unwrap().as_str().to_string();
            let (body, end_idx) = collect_block(&lines, i);
            ir.char_arrays.push(CharArray {
                name,
                body_raw: body,
                idx: seen_counter,
            });
            seen_counter += 1;
            i = end_idx + 1;
            continue;
        }

        if let Some(m) = re_static_int_array.captures(line) {
            let name = m.get(1).unwrap().as_str().to_string();
            let (body, end_idx) = collect_block(&lines, i);
            let vals: Vec<String> = body
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.trim_end_matches('\n').to_string())
                .collect();
            ir.int_arrays.push(IntArray {
                name,
                values: vals,
                idx: seen_counter,
            });
            seen_counter += 1;
            i = end_idx + 1;
            continue;
        }

        if let Some(m) = re_static_char_ptr_array.captures(line) {
            let name = m.get(1).unwrap().as_str().to_string();
            let (body, end_idx) = collect_block(&lines, i);
            let mut targets = Vec::new();
            for token in body.split(',') {
                let t = token.trim();
                if t.is_empty() {
                    continue;
                }
                if let Some(pos_amp) = t.find('&') {
                    let after = &t[pos_amp + 1..];
                    if let Some(bracket) = after.find('[') {
                        let ident = &after[..bracket];
                        if !ident.trim().is_empty() {
                            targets.push(ident.trim().to_string());
                        }
                    }
                } else if let Some(bracket) = t.find('[') {
                    let ident = &t[..bracket];
                    if !ident.trim().is_empty() {
                        targets.push(ident.trim().to_string());
                    }
                }
            }
            ir.pointer_arrays.push(PointerArray {
                name,
                targets,
                idx: seen_counter,
            });
            seen_counter += 1;
            i = end_idx + 1;
            continue;
        }

        i += 1;
    }

    ir
}

fn group_glyphs(ir: &FileIr) -> Vec<Glyph> {
    let mut map: HashMap<String, Glyph> = HashMap::new();

    // width/size defines
    for d in &ir.defines {
        if let Some(base) = d.name.strip_suffix("_width") {
            let g = map.entry(base.to_string()).or_default();
            g.base = base.to_string();
            g.order_key = g.order_key.min(d.idx);
            g.width = Some(d.clone());
            continue;
        }
        if let Some(base) = d.name.strip_suffix("_size") {
            let g = map.entry(base.to_string()).or_default();
            g.base = base.to_string();
            g.order_key = g.order_key.min(d.idx);
            g.size = Some(d.clone());
            continue;
        }
    }
    // arrays
    for a in &ir.char_arrays {
        let base = a.name.clone();
        let g = map.entry(base.clone()).or_default();
        g.base = base;
        g.order_key = g.order_key.min(a.idx);
        g.array = Some(a.clone());
    }

    let mut v: Vec<Glyph> = map.into_values().collect();
    v.sort_by_key(|g| g.order_key);
    v
}

fn render_rs(file_ir: &FileIr, basename: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    writeln!(out, "//! Converted from SDL_BGI C header").unwrap();
    writeln!(out, "//! Source: {basename}.h\n").unwrap();

    for c in &file_ir.header_comments {
        writeln!(out, "{c}").unwrap();
    }
    if !file_ir.header_comments.is_empty() {
        out.push('\n');
    }

    // Build glyph groups and render them together
    let glyphs = group_glyphs(file_ir);

    // Build a map for quick lookup by C name
    let mut define_map: HashMap<&str, &Define> = HashMap::new();
    for d in &file_ir.defines {
        define_map.insert(d.name.as_str(), d);
    }

    for g in glyphs {
        // Skip groups without the array — we only render complete glyphs or at least the array
        if g.array.is_none() {
            continue;
        }
        let base_c = g.base.as_str(); // e.g., "bold_2"
        let base_rs = upper_snake(base_c); // "BOLD_2"
        let width_rs = g.width.as_ref().map(|w| upper_snake(&w.name));
        let size_rs = g.size.as_ref().map(|s| upper_snake(&s.name));

        // Width first (if present)
        if let Some(wd) = &g.width {
            let wd_rs = width_rs.as_ref().unwrap();
            writeln!(out, "/// Width (pixels) for [`{base_rs}`] glyph data.").unwrap();
            writeln!(out, "#[doc(alias = \"{}\")]", wd.name).unwrap();
            writeln!(out, "pub const {wd_rs}: u8 = {};\n", wd.value).unwrap();
        }

        // Array next
        let arr = g.array.unwrap();
        // Decide array length: prefer size define, else compute
        let (len_expr, computed) = if let Some(sz) = &size_rs {
            (sz.clone(), false)
        } else {
            // Count entries
            let count = arr
                .body_raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .count();
            (count.to_string(), true)
        };

        match (&width_rs, &size_rs) {
            (Some(wr), Some(sr)) => {
                writeln!(out, "/// Glyph data; see [`{wr}`] and [`{sr}`].").unwrap();
            }
            (Some(wr), None) => {
                writeln!(out, "/// Glyph data; see [`{wr}`]. Length inferred.").unwrap();
            }
            (None, Some(sr)) => {
                writeln!(out, "/// Glyph data; see [`{sr}`].").unwrap();
            }
            (None, None) => {
                writeln!(out, "/// Glyph data. Length inferred.").unwrap();
            }
        }
        writeln!(out, "#[doc(alias = \"{}\")]", arr.name).unwrap();
        writeln!(out, "#[rustfmt::skip]").unwrap();
        if base_rs.ends_with("_WIDTH") {
            writeln!(out, "pub const {base_rs}: [u8; {len_expr}] = [").unwrap();
        } else {
            writeln!(out, "pub const {base_rs}: [i8; {len_expr} as usize] = [").unwrap();
        }
        // Keep original formatting
        let body = arr.body_raw.to_string();
        for line in body.lines() {
            let line: String = line
                .trim_end()
                .chars()
                .map(|c| c.to_ascii_uppercase())
                .collect();
            writeln!(out, "{line}").unwrap();
        }
        writeln!(out, "];").unwrap();
        if computed && !arr.body_raw.trim().is_empty() {
            writeln!(out, "// ^ length synthesized from element count").unwrap();
        }
        out.push('\n');

        // Size last (if present)
        if let Some(sz) = &g.size {
            let sz_rs = size_rs.as_ref().unwrap();
            writeln!(out, "/// Byte count for [`{base_rs}`] (number of entries).").unwrap();
            writeln!(out, "#[doc(alias = \"{}\")]", sz.name).unwrap();
            // Prefer numeric → usize; otherwise leave as-is
            if sz.value.chars().all(|c| c.is_ascii_digit()) {
                writeln!(out, "pub const {sz_rs}: u16 = {};\n", sz.value).unwrap();
            } else {
                writeln!(out, "pub const {sz_rs}: u16 = {value};\n", value = sz.value).unwrap();
            }
        }
    }

    // Emit any remaining defines that are NOT width/size (keep order)
    for d in &file_ir.defines {
        if d.name.ends_with("_width") || d.name.ends_with("_size") || d.name.ends_with("_NGLYPHS") {
            continue;
        }
        let rust_name = upper_snake(&d.name);
        writeln!(
            out,
            "#[doc(alias = \"{}\")]\npub const {rust_name}: i8 = {};\n",
            d.name, d.value
        )
        .unwrap();
    }

    // Aggregate int arrays (e.g., bold_width[], bold_size[])
    let mut int_arrays = file_ir.int_arrays.clone();
    int_arrays.sort_by_key(|a| a.idx);
    for a in &int_arrays {
        let rust_name = upper_snake(&a.name);
        writeln!(out, "#[doc(alias = \"{}\")]", a.name).unwrap();
        writeln!(out, "pub const {rust_name}: [u16; {}] = [", a.values.len()).unwrap();
        for v in &a.values {
            let rust_name = upper_snake(&v);
            writeln!(out, "    {rust_name},").unwrap();
        }
        writeln!(out, "];\n").unwrap();
    }

    // Aggregate pointer arrays (e.g., bold[])
    let mut ptr_arrays = file_ir.pointer_arrays.clone();
    ptr_arrays.sort_by_key(|p| p.idx);
    for p in &ptr_arrays {
        let rust_name = upper_snake(&p.name);
        writeln!(out, "#[doc(alias = \"{}\")]", p.name).unwrap();
        writeln!(
            out,
            "pub const {rust_name}: [&[i8]; {}] = [",
            p.targets.len()
        )
        .unwrap();
        for t in &p.targets {
            writeln!(out, "    &{},", upper_snake(t)).unwrap();
        }
        writeln!(out, "];\n").unwrap();
    }

    out
}

fn convert_file(path: &Path, out_dir: &Path) -> io::Result<()> {
    let stem = path.file_stem().unwrap().to_string_lossy();
    let text = fs::read_to_string(path)?;
    let ir = parse_h(&text);
    let rs = render_rs(&ir, &stem);
    let mut out_path = PathBuf::from(out_dir);
    out_path.push(format!("{}.rs", stem));
    let mut f = fs::File::create(&out_path)?;
    f.write_all(rs.as_bytes())?;
    eprintln!("Wrote {}", out_path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    let dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let dir_path = PathBuf::from(dir);
    if !dir_path.is_dir() {
        eprintln!("Not a directory: {}", dir_path.display());
        std::process::exit(2);
    }

    for entry in fs::read_dir(&dir_path)? {
        let entry = entry?;
        let p = entry.path();
        if p.extension().map(|e| e == "h").unwrap_or(false) {
            convert_file(&p, &dir_path)?;
        }
    }
    Ok(())
}
