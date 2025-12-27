use adw::prelude::*;
use relm4::{adw, gtk, prelude::FactoryComponent, RelmWidgetExt};
use stremio_core_losange::{stremio_core::types::resource::StreamSource, types::stream::Stream};

use crate::{common::format::Format, components::list::ListItemOutput};

pub struct StreamRow {
    pub title: String,
    pub description: String,
    pub external: bool,
}

#[relm4::factory(pub)]
impl FactoryComponent for StreamRow {
    type Input = ();
    type Output = ListItemOutput;
    type Init = Stream;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_height_request: 64,
            set_expand: true,
            set_focusable: true,

            add_controller = gtk::GestureClick {
                connect_released[sender, index] => move |_, _, _, _| {
                    sender.output_sender().emit(ListItemOutput::Clicked(index.current_index()));
                },
            },

            gtk::Box {
                set_margin_horizontal: 12,
                set_spacing: 12,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,
                    set_spacing: 3,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_single_line_mode: true,
                        set_label: &self.title.no_line_breaks(),
                    },

                    gtk::Label {
                        set_css_classes: &[relm4::css::classes::DIM_LABEL, relm4::css::classes::CAPTION],
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &self.description.no_line_breaks(),
                        set_single_line_mode: true,
                    },
                },

                gtk::Image {
                    set_width_request: 30,
                    set_icon_name: Some(match self.external {
                        true => "external-link",
                        false => "media-playback-start-symbolic",
                    }),
                },
            }
        },
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self {
            title: init.name,
            description: init.description,
            external: matches!(init.source, StreamSource::External { .. }),
        }
    }
}
