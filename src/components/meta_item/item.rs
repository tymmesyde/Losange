use relm4::gtk::gio::prelude::SettingsExt;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::gtk::{gdk, gio, glib};
use relm4::typed_view::grid::RelmGridItem;
use relm4::{adw, css, gtk, spawn_local, view, RelmWidgetExt};
use stremio_core_losange::types::item::{Item, Shape};
use tokio::sync::mpsc;
use url::Url;

use crate::common::image;
use crate::constants::{APP_ID, ITEM_MAX_SIZE, ITEM_MIN_SIZE};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetaItem {
    pub id: String,
    pub new_videos: usize,
    pub title: String,
    pub image: Option<Url>,
    pub shape: Shape,
}

impl From<&Item> for MetaItem {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id.clone(),
            new_videos: item.new_videos,
            title: item.name.clone(),
            image: item.image.clone(),
            shape: item.shape.clone(),
        }
    }
}

pub struct Widgets {
    container: gtk::Box,
    new_videos: gtk::Box,
    new_videos_label: gtk::Label,
    title: gtk::Label,
    icon: gtk::Image,
    info_container: adw::Clamp,
    image_container: gtk::Stack,
    image: gtk::Picture,
    info_title: gtk::Label,
    image_response_handle: Option<glib::JoinHandle<()>>,
    image_request_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RelmGridItem for MetaItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_expand: false,
                set_spacing: 12,

                #[name = "container"]
                gtk::Box {
                    add_css_class: css::classes::CARD,
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    set_overflow: gtk::Overflow::Hidden,

                    gtk::Overlay {
                        #[name = "new_videos"]
                        add_overlay = &gtk::Box {
                            add_css_class: "new-videos",
                            set_valign: gtk::Align::Start,
                            set_halign: gtk::Align::End,
                            set_margin_all: 8,

                            #[name = "new_videos_label"]
                            gtk::Label {
                                set_expand: true,
                            }
                        },

                        #[name = "revealer"]
                        add_overlay = &gtk::Revealer {
                            set_transition_type: gtk::RevealerTransitionType::Crossfade,

                            gtk::Box {
                                add_css_class: css::classes::OSD,
                                set_expand: true,
                                set_align: gtk::Align::Fill,

                                #[name = "title"]
                                gtk::Label {
                                    add_css_class: css::classes::TITLE_4,
                                    set_hexpand: true,
                                    set_margin_horizontal: 12,
                                    set_lines: 2,
                                    set_justify: gtk::Justification::Center,
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                                },

                                #[name = "icon"]
                                gtk::Image {
                                    set_align: gtk::Align::Center,
                                    set_hexpand: true,
                                    set_icon_name: Some("play"),
                                    set_icon_size: gtk::IconSize::Large,
                                }
                            }
                        },

                        #[name = "image_container"]
                        gtk::Stack {
                            set_transition_type: gtk::StackTransitionType::Crossfade,

                            add_child = &gtk::Image {
                                set_expand: true,
                                set_pixel_size: 30,
                                set_icon_name: Some("image-missing-symbolic"),
                            } -> {
                                set_name: "placeholder",
                            },

                            #[name = "image"]
                            add_child = &gtk::Picture {
                                set_content_fit: gtk::ContentFit::Cover,
                                set_expand: true,
                            } -> {
                                set_name: "image",
                            },
                        }
                    }
                },

                #[name = "info_container"]
                adw::Clamp {
                    #[name = "info_title"]
                    gtk::Label {
                        add_css_class: css::classes::HEADING,
                        set_justify: gtk::Justification::Center,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                    }
                }
            },
        }

        let controller = gtk::EventControllerMotion::new();

        let revealer_enter = revealer.clone();
        controller.connect_enter(move |_, _, _| {
            revealer_enter.set_reveal_child(true);
        });

        let revealer_leave = revealer.clone();
        controller.connect_leave(move |_| {
            revealer_leave.set_reveal_child(false);
        });

        container.add_controller(controller);

        let widgets = Widgets {
            container,
            new_videos,
            new_videos_label,
            title,
            icon,
            image_container,
            image,
            info_container,
            info_title,
            image_response_handle: None,
            image_request_handle: None,
        };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, root: &mut Self::Root) {
        let Widgets {
            container,
            new_videos,
            new_videos_label,
            title,
            icon,
            image_container,
            image,
            info_container,
            info_title,
            image_response_handle,
            image_request_handle,
        } = widgets;

        let settings = gio::Settings::new(APP_ID);

        let (width, height) = match self.shape {
            Shape::Poster => (ITEM_MIN_SIZE, ITEM_MAX_SIZE),
            Shape::Square => (ITEM_MIN_SIZE, ITEM_MIN_SIZE),
            Shape::Landscape => (ITEM_MIN_SIZE, ITEM_MIN_SIZE),
        };

        root.set_width_request(width);
        container.set_size_request(width, height);

        new_videos.set_visible(self.new_videos > 0);
        new_videos_label.set_label(format!("+{}", self.new_videos).as_str());

        title.set_label(&self.title);
        title.set_visible(!settings.boolean("content-title-below"));
        icon.set_visible(settings.boolean("content-title-below"));

        let image_container_widget = image_container.clone();
        let image_widget = image.clone();

        if let Some(uri) = self.image.clone() {
            let (tx, mut rx) = mpsc::channel::<gdk::MemoryTexture>(1);

            let response_handle = spawn_local(async move {
                if let Some(texture) = rx.recv().await {
                    image_widget.set_paintable(Some::<&gdk::MemoryTexture>(texture.as_ref()));
                    image_container_widget.set_visible_child_name("image");
                }
            });

            let request_handle = tokio::spawn(async move {
                if let Ok(texture) = image::load_as_texture(uri, (width, height)).await {
                    tx.send(texture).await.ok();
                }
            });

            image_response_handle.replace(response_handle);
            image_request_handle.replace(request_handle);
        }

        info_container.set_maximum_size(width);
        info_container.set_visible(settings.boolean("content-title-below"));
        info_title.set_label(&self.title);
    }

    fn unbind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            image_container,
            image,
            image_response_handle,
            image_request_handle,
            ..
        } = widgets;

        if let Some(handle) = image_response_handle.take() {
            handle.abort();
        }

        if let Some(handle) = image_request_handle.take() {
            handle.abort();
        }

        image_container.set_visible_child_name("placeholder");
        image.set_paintable(None::<&gdk::MemoryTexture>);
    }
}
