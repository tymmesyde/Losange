use adw::prelude::*;
use relm4::{adw, factory, gtk, prelude::FactoryComponent, RelmWidgetExt};
use stremio_core_losange::types::video::Video;

use crate::{
    common::format::Format,
    components::list::{ListItemInput, ListItemOutput},
};

pub struct VideoRow {
    pub title: String,
    pub description: String,
    pub episode: u32,
    pub visible: bool,
}

#[factory(pub)]
impl FactoryComponent for VideoRow {
    type Input = ListItemInput;
    type Output = ListItemOutput;
    type Init = Video;
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

                #[watch]
                set_visible: self.visible,

                gtk::Label {
                    set_width_request: 26,
                    set_margin_end: 4,
                    set_label: &self.episode.to_string(),
                    set_visible: self.episode.gt(&0),
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,
                    set_spacing: 3,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_single_line_mode: true,
                        set_label: &self.title,
                    },

                    gtk::Label {
                        set_css_classes: &[relm4::css::classes::DIM_LABEL, relm4::css::classes::CAPTION],
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &self.description.no_line_breaks(),
                        set_visible: !self.description.is_empty(),
                    },
                },

                gtk::Image {
                    set_width_request: 30,
                    set_icon_name: Some("right"),
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
            episode: init
                .series_info
                .map_or(0, |series_info| series_info.episode),
            visible: false,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::FactorySender<Self>) {
        match message {
            ListItemInput::Show => self.visible = true,
            ListItemInput::Hide => self.visible = false,
        }
    }
}
