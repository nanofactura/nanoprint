//! How a site presents itself: an optional name and logo that the page sends in the connection
//! setup (see `docs/integration.md`). Both are claims made by the page, so the origin stays what
//! the user decides on; they only make the prompt and the panel easier to recognize.
//!
//! The name keeps letters, digits, spaces and common punctuation; everything else is dropped. The
//! logo must be an inline PNG, JPEG or WebP image (a `data:` URL): it is decoded here, in
//! memory-safe code with size limits, and re-encoded as a small PNG, so the webviews never parse
//! what the page sent and nothing is ever fetched from the network or read from disk. SVG is not
//! accepted: its references (`<use>`, masks, patterns, markers…) let a few kilobytes demand
//! unbounded rendering work, and pages can rasterize their SVG logo themselves.

use std::io::Cursor;

use base64::{Engine, engine::general_purpose::STANDARD};
use icu_normalizer::ComposingNormalizerBorrowed;
use image::{DynamicImage, ImageFormat, ImageReader, Limits, RgbaImage, imageops};
use serde_json::Value;

/// Side of the stored logo, in pixels: sharp at 48 CSS pixels on 2x screens.
pub const ICON_SIZE: u32 = 96;
/// Largest logo accepted, before base64.
const MAX_ICON_BYTES: usize = 32 * 1024;
/// Largest logo accepted, per side: decoding cost is bounded by the pixel count.
const MAX_ICON_SIDE: u32 = 1024;
const MAX_NAME_CHARS: usize = 40;
/// Characters of the raw name that are looked at; the rest is ignored unread.
const MAX_RAW_NAME_CHARS: usize = 256;
/// Punctuation kept in names, besides letters, digits and spaces. No `:`, `/` or `@`: a name must
/// not read like an address (`https://bank.example`).
const NAME_PUNCTUATION: &[char] = &[
    '.', ',', '-', '_', '&', '\'', '’', '(', ')', '!', '?', '+', '#',
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identity {
    pub name: Option<String>,
    /// A PNG of [`ICON_SIZE`] pixels square.
    pub icon: Option<Vec<u8>>,
}

impl Identity {
    /// Reads `{"nanoprint": {"name", "icon"}}` from the setup message's `certificate` field.
    /// Anything else there (nothing, or a QZ Tray certificate) gives `None`, as do values that
    /// fail validation: a site without a name or logo is shown by its origin alone.
    pub fn from_certificate(certificate: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(certificate).ok()?;
        let claims = value.get("nanoprint")?;
        let name = claims
            .get("name")
            .and_then(Value::as_str)
            .and_then(sanitize_name);
        let icon =
            claims
                .get("icon")
                .and_then(Value::as_str)
                .and_then(|icon| match decode_icon(icon) {
                    Ok(png) => Some(png),
                    Err(reason) => {
                        log::debug!("ignoring a site's logo: {reason}");
                        None
                    }
                });
        (name.is_some() || icon.is_some()).then_some(Self { name, icon })
    }
}

/// A stored logo as the windows show it.
pub fn icon_data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(png))
}

/// Keeps letters, digits, single spaces and [`NAME_PUNCTUATION`], up to [`MAX_NAME_CHARS`]: no
/// control or invisible characters, no text direction overrides, no symbols or emoji.
fn sanitize_name(raw: &str) -> Option<String> {
    let raw: String = raw.chars().take(MAX_RAW_NAME_CHARS).collect();
    let normalized = ComposingNormalizerBorrowed::new_nfc().normalize(&raw);
    let mut name = String::new();
    let mut length = 0;
    let mut space = false;
    for c in normalized.chars() {
        if c.is_whitespace() {
            space = !name.is_empty();
        } else if c.is_alphanumeric() || NAME_PUNCTUATION.contains(&c) {
            let needed = if space { 2 } else { 1 };
            if length + needed > MAX_NAME_CHARS {
                break;
            }
            if space {
                name.push(' ');
                space = false;
            }
            name.push(c);
            length += needed;
        }
    }
    name.chars().any(char::is_alphanumeric).then_some(name)
}

/// Decodes a `data:image/{png,jpeg,webp};base64,…` logo into a PNG of [`ICON_SIZE`].
fn decode_icon(data_url: &str) -> Result<Vec<u8>, &'static str> {
    let (mime, payload) = data_url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or("not a base64 data URL")?;
    if payload.len() > MAX_ICON_BYTES.div_ceil(3) * 4 {
        return Err("larger than 32 KiB");
    }
    let format = match mime.to_ascii_lowercase().as_str() {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        "image/webp" => ImageFormat::WebP,
        _ => return Err("unsupported image type"),
    };
    let bytes = STANDARD.decode(payload).map_err(|_| "invalid base64")?;
    if image::guess_format(&bytes).ok() != Some(format) {
        return Err("the content does not match the declared type");
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_ICON_SIDE);
    limits.max_image_height = Some(MAX_ICON_SIDE);
    limits.max_alloc = Some(16 * 1024 * 1024);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| "could not decode the image")?;
    let mut png = Vec::new();
    fit(&image)
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|_| "could not encode the logo")?;
    Ok(png)
}

/// Scales the image to fit [`ICON_SIZE`] and centers it on a transparent square.
fn fit(image: &DynamicImage) -> RgbaImage {
    let scaled = image
        .resize(ICON_SIZE, ICON_SIZE, imageops::FilterType::Lanczos3)
        .to_rgba8();
    let mut square = RgbaImage::new(ICON_SIZE, ICON_SIZE);
    imageops::overlay(
        &mut square,
        &scaled,
        i64::from((ICON_SIZE - scaled.width()) / 2),
        i64::from((ICON_SIZE - scaled.height()) / 2),
    );
    square
}

#[cfg(test)]
mod tests {
    use super::*;

    fn certificate(name: &str, icon: &str) -> String {
        serde_json::json!({ "nanoprint": { "name": name, "icon": icon } }).to_string()
    }

    fn data_url(mime: &str, bytes: &[u8]) -> String {
        format!("data:{mime};base64,{}", STANDARD.encode(bytes))
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        RgbaImage::from_pixel(width, height, image::Rgba([200, 30, 30, 255]))
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    fn decoded(png: &[u8]) -> RgbaImage {
        image::load_from_memory_with_format(png, ImageFormat::Png)
            .unwrap()
            .to_rgba8()
    }

    #[test]
    fn keeps_only_safe_characters_in_names() {
        assert_eq!(
            sanitize_name("  Kiosco   Don Pepe  ").as_deref(),
            Some("Kiosco Don Pepe")
        );
        // Text direction override, zero-width space, control characters and emoji.
        assert_eq!(
            sanitize_name("Shop\u{202E}moc.evil\u{200B}\u{7}🙂").as_deref(),
            Some("Shopmoc.evil")
        );
        assert_eq!(
            sanitize_name("<b>Café & Co.</b>").as_deref(),
            Some("bCafé & Co.b")
        );
        // A decomposed "é" is composed rather than losing its accent.
        assert_eq!(sanitize_name("Cafe\u{301}").as_deref(), Some("Café"));
        assert_eq!(sanitize_name("🙂 -- 🙂"), None);
        // A name cannot pose as another site's address.
        assert_eq!(
            sanitize_name("https://bank.example").as_deref(),
            Some("httpsbank.example")
        );
    }

    #[test]
    fn limits_names_to_forty_characters() {
        let count = |raw: &str| sanitize_name(raw).unwrap().chars().count();
        assert_eq!(count(&"x".repeat(100)), 40);
        // A space and a letter added together must not step over the limit.
        assert_eq!(
            count(&format!("{} {}", "x".repeat(39), "y".repeat(10_000))),
            39
        );
        assert_eq!(
            count(&format!("{} {}", "x".repeat(38), "y".repeat(10_000))),
            40
        );
        assert_eq!(count(&"ab ".repeat(10_000)), 40);
    }

    #[test]
    fn ignores_anything_but_the_nanoprint_claims() {
        assert_eq!(Identity::from_certificate("null"), None);
        assert_eq!(
            Identity::from_certificate("-----BEGIN CERTIFICATE-----\nMIIB…"),
            None
        );
        assert_eq!(Identity::from_certificate(r#"{"name": "x"}"#), None);
        assert_eq!(
            Identity::from_certificate(&certificate("Kiosco", "https://tracker.example/logo.png")),
            Some(Identity {
                name: Some("Kiosco".into()),
                icon: None
            })
        );
    }

    #[test]
    fn reencodes_logos_as_small_squares() {
        let identity = Identity::from_certificate(&certificate(
            "Kiosco",
            &data_url("image/png", &png(300, 150)),
        ))
        .unwrap();
        let icon = decoded(&identity.icon.unwrap());
        assert_eq!(icon.dimensions(), (ICON_SIZE, ICON_SIZE));
        // Letterboxed: transparent above, the logo in the middle.
        assert_eq!(icon.get_pixel(48, 2)[3], 0);
        assert_eq!(icon.get_pixel(48, 48)[3], 255);
    }

    #[test]
    fn refuses_logos_that_are_too_large_mislabeled_or_svg() {
        assert!(decode_icon(&data_url("image/png", &png(1500, 10))).is_err());
        assert!(decode_icon(&data_url("image/jpeg", &png(10, 10))).is_err());
        assert!(decode_icon(&data_url("image/gif", b"GIF89a")).is_err());
        assert!(decode_icon(&data_url("image/png", &vec![0; MAX_ICON_BYTES + 1])).is_err());
        assert!(decode_icon("data:image/png,raw").is_err());
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#;
        assert!(decode_icon(&data_url("image/svg+xml", svg)).is_err());
    }
}
