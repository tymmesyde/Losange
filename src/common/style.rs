use gtk::gdk;
use itertools::Itertools;
use relm4::gtk::{self, glib::object::ObjectExt, InterfaceColorScheme};

pub fn create_css_provider() -> gtk::CssProvider {
    let css_provider = gtk::CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_SETTINGS,
        );
    }

    css_provider
}

pub fn colors_to_css(colors: Vec<String>) -> String {
    colors
        .iter()
        .enumerate()
        .map(|(i, color)| format!("@define-color background_color_{} {};", i, color))
        .join("")
}

pub fn has_dark_theme() -> bool {
    gtk::Settings::default().is_some_and(|settings| {
        settings.property::<InterfaceColorScheme>("gtk-interface-color-scheme")
            == InterfaceColorScheme::Dark
    })
}

pub trait ColorHexExt {
    fn to_hex(&self) -> String;
    fn parse_hex(hex: &str) -> Result<Self, String>
    where
        Self: Sized;
}

impl ColorHexExt for gdk::RGBA {
    fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            (self.alpha() * 255.0) as u8,
            (self.red() * 255.0) as u8,
            (self.green() * 255.0) as u8,
            (self.blue() * 255.0) as u8,
        )
    }

    fn parse_hex(hex: &str) -> Result<Self, String> {
        let parse = |offset: usize| -> Result<f32, String> {
            u8::from_str_radix(&hex[offset..offset + 2], 16)
                .map(|value| value as f32 / 255.0)
                .map_err(|e| e.to_string())
        };

        let (a, r, g, b) = match hex.len() {
            9 => (parse(1)?, parse(3)?, parse(5)?, parse(7)?),
            7 => (1.0, parse(1)?, parse(3)?, parse(5)?),
            _ => (1.0, 1.0, 1.0, 1.0),
        };

        Ok(gdk::RGBA::new(r, g, b, a))
    }
}
