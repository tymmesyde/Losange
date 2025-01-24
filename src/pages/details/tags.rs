use gtk::prelude::*;
use relm4::{css, gtk, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for Tags {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            #[name = "label"]
            gtk::Label {
                set_css_classes: &[css::classes::TITLE_4, css::classes::DIM_LABEL],
                set_halign: gtk::Align::Start,
            },
        },
    }
}
