use std::marker::PhantomData;

use gtk::gio;
use gtk::prelude::*;
use relm4::{
    adw,
    component::{AsyncComponent, AsyncComponentController, AsyncController},
    css,
    factory::{DynamicIndex, FactoryComponent, FactoryView, Position},
    gtk, FactorySender, RelmWidgetExt,
};
use stremio_core_losange::{stremio_core::types::resource::PosterShape, types::item::Item};

use crate::{
    app::AppMsg,
    components::image::{init::ImageInit, Image, ImageInput},
    constants::{APP_ID, ITEM_MAX_SIZE, ITEM_MIN_SIZE},
    APP_BROKER,
};

#[derive(Debug, Clone)]
pub enum ItemBoxInput {
    LoadImage,
    Show,
    Hide,
    Hover(bool),
    Clicked,
}

#[derive(Debug)]
pub struct ItemBox<P: FactoryView + 'static> {
    phantom: PhantomData<P>,
    settings: gio::Settings,
    pub id: String,
    pub r#type: String,
    pub image: AsyncController<Image>,
    pub size: (i32, i32),
    pub title: String,
    pub new_videos: usize,
    pub hover: bool,
    pub visible: bool,
}

#[relm4::factory(pub)]
impl<P: FactoryView + 'static> FactoryComponent for ItemBox<P>
where
    ItemBox<P>: Position<<P as FactoryView>::Position, DynamicIndex>,
    gtk::Box: AsRef<<P as FactoryView>::Children>,
{
    type Input = ItemBoxInput;
    type Output = ();
    type Init = Item;
    type CommandOutput = ();
    type ParentWidget = P;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_width_request: self.size.1,
            set_spacing: 12,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                set_height_request: self.size.0,
                set_width_request: self.size.1,

                adw::Clamp {
                    set_maximum_size: self.size.1,

                    gtk::Button {
                        add_css_class: css::classes::CARD,
                        set_expand: true,
                        set_align: gtk::Align::Fill,
                        set_overflow: gtk::Overflow::Hidden,

                        #[watch]
                        set_visible: self.visible,

                        connect_clicked => ItemBoxInput::Clicked,

                        add_controller = gtk::EventControllerMotion {
                            connect_enter[sender] => move |_event, _x, _y| {
                                sender.input(ItemBoxInput::Hover(true));
                            },
                            connect_leave => ItemBoxInput::Hover(false),
                        },

                        gtk::Overlay {
                            add_overlay = &gtk::Box {
                                add_css_class: "new-videos",
                                set_valign: gtk::Align::Start,
                                set_halign: gtk::Align::End,
                                set_margin_all: 8,
                                set_visible: self.new_videos > 0,

                                gtk::Label {
                                    set_expand: true,
                                    set_label: format!("+{}", self.new_videos).as_str(),
                                }
                            },

                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: self.hover,

                                gtk::Box {
                                    add_css_class: css::classes::OSD,
                                    set_expand: true,
                                    set_align: gtk::Align::Fill,

                                    gtk::Label {
                                        add_css_class: css::classes::TITLE_4,
                                        set_hexpand: true,
                                        set_margin_horizontal: 12,
                                        set_lines: 2,
                                        set_justify: gtk::Justification::Center,
                                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                                        set_label: &self.title,

                                        #[watch]
                                        set_visible: !self.settings.boolean("content-title-below"),
                                    },

                                    gtk::Image {
                                        set_align: gtk::Align::Center,
                                        set_hexpand: true,
                                        set_icon_name: Some("play"),
                                        set_icon_size: gtk::IconSize::Large,

                                        #[watch]
                                        set_visible: self.settings.boolean("content-title-below"),
                                    }
                                }
                            },

                            self.image.widget(),
                        }
                    },
                }
            },

            adw::Clamp {
                set_maximum_size: self.size.1,

                #[watch]
                set_visible: self.settings.boolean("content-title-below"),

                gtk::Label {
                    add_css_class: css::classes::HEADING,
                    set_justify: gtk::Justification::Center,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_label: &self.title,
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let settings = gio::Settings::new(APP_ID);

        let size = match init.shape {
            PosterShape::Poster => (ITEM_MAX_SIZE, ITEM_MIN_SIZE),
            PosterShape::Square => (ITEM_MIN_SIZE, ITEM_MIN_SIZE),
            PosterShape::Landscape => (ITEM_MIN_SIZE, ITEM_MIN_SIZE),
        };

        let image = Image::builder()
            .launch(
                ImageInit::builder()
                    .source(init.image)
                    .preload(false)
                    .size(size)
                    .content_fit(gtk::ContentFit::Cover)
                    .placeholder("image-missing-symbolic")
                    .build(),
            )
            .detach();

        Self {
            phantom: PhantomData,
            settings,
            id: init.id,
            r#type: init.r#type,
            image,
            size,
            title: init.name,
            new_videos: init.new_videos,
            hover: false,
            visible: false,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ItemBoxInput::LoadImage => self.image.emit(ImageInput::Load),
            ItemBoxInput::Show => self.visible = true,
            ItemBoxInput::Hide => self.visible = false,
            ItemBoxInput::Hover(state) => self.hover = state,
            ItemBoxInput::Clicked => APP_BROKER.send(AppMsg::OpenDetails((
                self.id.to_owned(),
                self.r#type.to_owned(),
            ))),
        }
    }
}
