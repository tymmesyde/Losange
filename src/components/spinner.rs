use gtk::prelude::*;
use relm4::{adw, gtk, RelmWidgetExt, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for Spinner {
    view! {
        adw::Spinner {
            set_expand: true,
            set_halign: gtk::Align::Center,
        }
    }
}
