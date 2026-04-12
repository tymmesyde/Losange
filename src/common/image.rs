use std::io::Cursor;

use color_thief::{get_palette, ColorFormat};
use gtk::gdk::{MemoryFormat, MemoryTexture};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use hsl::HSL;
use image::ImageReader;
use itertools::Itertools;
use relm4::gtk;

pub fn load_as_texture(bytes: bytes::Bytes, size: (i32, i32)) -> anyhow::Result<MemoryTexture> {
    let cursor = Cursor::new(&bytes);
    let img = ImageReader::new(cursor).with_guessed_format()?.decode()?;

    let img = if img.width() > size.0 as u32 || img.height() > size.1 as u32 {
        img.thumbnail(size.0 as u32, size.1 as u32)
    } else {
        img
    };

    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let stride = width * 4;

    let raw_data = img.into_raw();
    let data = glib::Bytes::from_owned(raw_data);
    let texture = MemoryTexture::new(
        width as i32,
        height as i32,
        MemoryFormat::R8g8b8a8,
        &data,
        stride as usize,
    );

    Ok(texture)
}

pub fn pixbuf_from_bytes<T: AsRef<[u8]> + Send + 'static>(bytes: T) -> Result<Pixbuf, glib::Error> {
    let cursor = std::io::Cursor::new(bytes);
    Pixbuf::from_read(cursor)
}

pub fn colors_from_bytes(
    bytes: &[u8],
    dark_theme: bool,
) -> Result<Vec<String>, color_thief::Error> {
    let palette = get_palette(bytes, ColorFormat::Rgb, 1, 2)?;

    Ok(palette
        .iter()
        .map(|rgb| HSL::from_rgb(&[rgb.r, rgb.g, rgb.b]))
        .map(|mut hsl| {
            hsl.l = hsl.l.clamp(
                if dark_theme { 0.0 } else { 0.8 },
                if dark_theme { 0.2 } else { 1.0 },
            );
            hsl.s = hsl.s.clamp(
                if dark_theme { 0.0 } else { 0.4 },
                if dark_theme { 0.6 } else { 1.0 },
            );
            hsl
        })
        .map(|hsl| hsl.to_rgb())
        .map(|(r, g, b)| format!("rgb({r},{g},{b})"))
        .collect_vec())
}
