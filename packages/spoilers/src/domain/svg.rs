use regex::{Captures, Regex};
use std::sync::OnceLock;

static FILL_RE: OnceLock<Regex> = OnceLock::new();
static STROKE_RE: OnceLock<Regex> = OnceLock::new();

pub fn recolour(svg: &str, colour: &str) -> String {
    let fill = FILL_RE.get_or_init(|| Regex::new(r#"fill="[^"]*""#).unwrap());
    let stroke = STROKE_RE.get_or_init(|| Regex::new(r#"stroke="[^"]*""#).unwrap());


    let svg = svg.replacen("<svg ", &format!(r#"<svg fill="{colour}" stroke="{colour}" "#), 1);

    let recoloured = fill.replace_all(&svg, |_: &Captures| format!(r#"fill="{colour}""#));
    let recoloured = stroke.replace_all(&recoloured, |_: &Captures| format!(r#"stroke="{colour}""#));

    recoloured.into_owned()
}


pub fn to_png(svg: &str) -> Vec<u8> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &options).unwrap();

    let size = tree.size().to_int_size();
    let scale = 128.0 / size.width().max(size.height()) as f32;

    let w = (size.width() as f32 * scale) as u32;
    let h = (size.height() as f32 * scale) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();

    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    pixmap.encode_png().unwrap()
}