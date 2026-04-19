use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, OrientableExt, WidgetExt},
    },
    typed_view::list::RelmListItem,
    view, RelmWidgetExt,
};

use crate::common::format::Format;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ListItem {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub icon: &'static str,
}

pub struct Widgets {
    number: gtk::Label,
    title: gtk::Label,
    description: gtk::Label,
    icon: gtk::Image,
}

impl RelmListItem for ListItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        view! {
            root = gtk::Box {
                set_height_request: 64,
                set_expand: true,
                set_focusable: true,

                gtk::Box {
                    set_margin_horizontal: 12,
                    set_spacing: 12,

                    #[name = "number"]
                    gtk::Label {
                        set_width_request: 26,
                        set_margin_end: 4,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Center,
                        set_hexpand: true,
                        set_spacing: 3,

                        #[name = "title"]
                        gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_single_line_mode: true,
                        },

                        #[name = "description"]
                        gtk::Label {
                            set_css_classes: &[relm4::css::classes::DIM_LABEL, relm4::css::classes::CAPTION],
                            set_halign: gtk::Align::Start,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_single_line_mode: true,
                        },
                    },

                    #[name = "icon"]
                    gtk::Image {
                        set_width_request: 30,
                    },
                }
            },
        }

        let widgets = Widgets {
            number,
            title,
            description,
            icon,
        };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            number,
            title,
            description,
            icon,
        } = widgets;

        number.set_label(&self.number.to_string());
        number.set_visible(self.number.gt(&0));

        title.set_label(&self.title);
        description.set_label(&self.description.no_line_breaks());
        description.set_visible(!self.description.is_empty());

        icon.set_icon_name(Some(self.icon));
    }
}
