use regex::{Captures, Regex};
use std::sync::OnceLock;

static FILL_RE: OnceLock<Regex> = OnceLock::new();
static STROKE_RE: OnceLock<Regex> = OnceLock::new();

/// # Panics
/// Panics if the internal fill or stroke regex patterns fail to compile (unreachable in practice).
pub fn recolour(svg: &str, colour: &str) -> String {
    let fill = FILL_RE.get_or_init(|| Regex::new(r#"fill="[^"]*""#).expect("Invalid regex"));
    let stroke = STROKE_RE.get_or_init(|| Regex::new(r#"stroke="[^"]*""#).expect("Invalid regex"));

    let svg = svg.replacen(
        "<svg ",
        &format!(r#"<svg fill="{colour}" stroke="{colour}" "#),
        1,
    );

    let recoloured = fill.replace_all(&svg, |_: &Captures| format!(r#"fill="{colour}""#));
    let recoloured =
        stroke.replace_all(&recoloured, |_: &Captures| format!(r#"stroke="{colour}""#));

    recoloured.into_owned()
}

/// # Panics
/// Panics if the parsed SVG has zero-width or zero-height dimensions, causing a division by zero
/// when computing the scale. Valid SVGs parsed by `usvg` will not have zero dimensions.
#[must_use]
#[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
pub fn to_png(svg: &str) -> Option<Vec<u8>> {
    let options = usvg::Options::default();
    let tree = match usvg::Tree::from_str(svg, &options) {
        Ok(tree) => tree,
        Err(why) => {
            log::error!("{why}");
            return None;
        }
    };

    let size = tree.size().to_int_size();
    let max_dim = size.width().max(size.height());
    let w = size.width() * 128 / max_dim;
    let h = size.height() * 128 / max_dim;
    let scale = 128.0_f32 / max_dim as f32;

    let Some(mut pixmap) = tiny_skia::Pixmap::new(w, h) else {
        log::error!("Failed to create pixmap");
        return None;
    };

    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    let png = match pixmap.encode_png() {
        Ok(png) => png,
        Err(why) => {
            log::error!("Failed to encode png: {why}");
            return None;
        }
    };

    Some(png)
}
