use gtk::prelude::*;
use relm4::{adw, gtk, RelmWidgetExt, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for Spinner {
    view! {
        adw::Spinner {
            set_expand: true,
            set_halign: gtk::Align::Center,
            set_width_request: 25,
            set_height_request: 25,
        }
    }
}
