/// Lossy quality for stored card images. Chosen by eye: no visible difference
/// from Scryfall's PNGs even at 80, so 85 leaves margin.
pub const WEBP_QUALITY: f32 = 85.0;
/// libwebp's default effort. Method 6 is ~15x slower for ~5% smaller files.
pub const WEBP_METHOD: i32 = 4;

#[derive(Debug, thiserror::Error)]
pub enum WebpError {
    #[error("decode: {0}")]
    Decode(#[from] image::ImageError),
    #[error("encode: {0}")]
    Encode(String),
}

/// Re-encodes a downloaded card image as lossy WebP, keeping the alpha
/// channel (Scryfall card PNGs have transparent rounded corners).
///
/// The input format is detected from its bytes: usually PNG, but Scryfall
/// serves JPEGs at some `png` URLs (low-res scans).
///
/// # Errors
/// Returns [`WebpError::Decode`] if `bytes` isn't a decodable PNG or JPEG
/// (including empty input) and [`WebpError::Encode`] if libwebp rejects the
/// image.
pub fn to_webp(bytes: &[u8]) -> Result<Vec<u8>, WebpError> {
    let rgba = image::load_from_memory(bytes)?.to_rgba8();

    let mut config = ::webp::WebPConfig::new()
        .map_err(|()| WebpError::Encode(String::from("failed to initialise WebPConfig")))?;
    config.quality = WEBP_QUALITY;
    config.method = WEBP_METHOD;
    config.lossless = 0;

    let encoded = ::webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height())
        .encode_advanced(&config)
        .map_err(|e| WebpError::Encode(format!("{e:?}")))?;

    Ok(encoded.to_vec())
}

#[cfg(test)]
pub(crate) fn fixture_png() -> Vec<u8> {
    let mut img = image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
    img.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[cfg(test)]
pub(crate) fn fixture_jpeg() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(32, 24, image::Rgb([0, 128, 255]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_webp(bytes: &[u8]) -> image::DynamicImage {
        image::load_from_memory_with_format(bytes, image::ImageFormat::WebP).unwrap()
    }

    #[test]
    fn produces_webp_with_same_dimensions() {
        let out = to_webp(&fixture_png()).unwrap();
        assert_eq!(&out[0..4], b"RIFF");
        assert_eq!(&out[8..12], b"WEBP");
        let img = decode_webp(&out);
        assert_eq!((img.width(), img.height()), (64, 64));
    }

    #[test]
    fn keeps_alpha() {
        let img = decode_webp(&to_webp(&fixture_png()).unwrap()).to_rgba8();
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        assert_eq!(img.get_pixel(32, 32)[3], 255);
    }

    #[test]
    fn converts_jpeg_input() {
        // Scryfall serves JPEGs at some `png` URLs (low-res scans); ~1.3% of
        // the existing library are JPEGs named `.png`.
        let img = decode_webp(&to_webp(&fixture_jpeg()).unwrap());
        assert_eq!((img.width(), img.height()), (32, 24));
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(to_webp(&[]), Err(WebpError::Decode(_))));
    }

    #[test]
    fn rejects_non_png() {
        assert!(matches!(to_webp(b"not a png"), Err(WebpError::Decode(_))));
    }
}
