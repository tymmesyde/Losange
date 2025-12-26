use gtk::gdk;
use itertools::Itertools;
use relm4::gtk::{self, glib::object::ObjectExt};

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
    let settings = gtk::Settings::default();
    match settings {
        Some(settings) => settings.property::<bool>("gtk-application-prefer-dark-theme"),
        None => false,
    }
}
