use color_thief::{get_palette, ColorFormat};
use hsl::HSL;
use itertools::Itertools;
use relm4::gtk::gdk_pixbuf::{InterpType, Pixbuf};
use relm4::gtk::glib;

pub fn pixbuf_from_bytes<T: AsRef<[u8]> + Send + 'static>(bytes: T) -> Result<Pixbuf, glib::Error> {
    let cursor = std::io::Cursor::new(bytes);
    Pixbuf::from_read(cursor)
}

pub fn scale_pixbuf(pixbuf: Pixbuf, (width, height): (i32, i32)) -> Option<Pixbuf> {
    let orig_width = pixbuf.width();
    let orig_height = pixbuf.height();

    let aspect_ratio = orig_width as f64 / orig_height as f64;

    let (new_width, new_height) =
        if orig_width as f64 / width as f64 > orig_height as f64 / width as f64 {
            (width, (width as f64 / aspect_ratio).round() as i32)
        } else {
            ((height as f64 * aspect_ratio).round() as i32, height)
        };

    pixbuf.scale_simple(new_width, new_height, InterpType::Bilinear)
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
