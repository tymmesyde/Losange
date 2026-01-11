mod button;

use adw::gtk::ScrolledWindow;
use adw::prelude::*;
use button::{Button, ButtonInit, ButtonOutput};
use gtk::gio;
use relm4::factory::{FactoryVecDeque, FactoryView};
use relm4::{css, prelude::*};
use relm4::{gtk, FactorySender};
use rust_i18n::t;
use stremio_core_losange::stremio_core::types::addon::ResourceRequest;
use stremio_core_losange::types::catalog::Catalog;

use crate::app::AppMsg;
use crate::common::layout::{self, ScrollPosition};
use crate::components::image::{init::ImageInit, Image};
use crate::components::item_box::{ItemBox, ItemBoxInput};
use crate::constants::{APP_ID, CATALOG_ICON_SIZE, ITEM_MAX_SIZE, ITEM_MIN_SIZE};
use crate::APP_BROKER;

pub type CatalogRowInit = Catalog;

#[derive(Debug, Clone)]
pub enum CatalogRowInput {
    ScrollLayout,
    Show,
    Hide,
    Hover(bool),
    ScrollLeft,
    ScrollRight,
    ShowAllClicked,
}

pub struct CatalogRow {
    settings: gio::Settings,
    index: usize,
    hover: bool,
    visible: bool,
    size: i32,
    request: Option<ResourceRequest>,
    icon: AsyncController<Image>,
    r#type: Option<String>,
    addon_name: Option<String>,
    name: Option<String>,
    scroll_posiiton: ScrollPosition,
    scrolled_window: ScrolledWindow,
    left_button: Controller<Button>,
    right_button: Controller<Button>,
    pub items: FactoryVecDeque<ItemBox<gtk::Box>>,
}

#[relm4::factory(pub)]
impl FactoryComponent for CatalogRow {
    type Init = CatalogRowInit;
    type Input = CatalogRowInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_vertical: 3,
            set_spacing: 12,

            gtk::Box {
                set_margin_horizontal: 16,

                gtk::Box {
                    set_hexpand: true,
                    set_spacing: 12,

                    gtk::Box {
                        set_spacing: 8,

                        gtk::Box {
                            add_css_class: "small-icon",
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_expand: false,
                            set_overflow: gtk::Overflow::Hidden,

                            #[watch]
                            set_visible: self.settings.boolean("catalog-addon-icon"),

                            self.icon.widget(),
                        },

                        gtk::Label {
                            set_css_classes: &[css::classes::TITLE_4, css::classes::DIM_LABEL],
                            set_valign: gtk::Align::Center,
                            set_label: self.addon_name.as_deref().unwrap_or_default(),

                            #[watch]
                            set_visible: self.addon_name.is_some() && self.settings.boolean("catalog-addon-name"),
                        },

                        gtk::Label {
                            add_css_class: css::classes::TITLE_4,
                            set_valign: gtk::Align::Center,
                            set_label: self.name.as_ref().unwrap_or(&t!("continue_watching").to_string()),
                        }
                    },

                    gtk::Button {
                        add_css_class: "small-tag",
                        set_valign: gtk::Align::Center,
                        set_margin_top: 1,
                        set_visible: self.r#type.is_some(),
                        set_label: &t!(self.r#type.as_deref().unwrap_or_default()),
                    }
                },

                gtk::Button {
                    add_css_class: css::classes::FLAT,
                    connect_clicked => CatalogRowInput::ShowAllClicked,

                    #[watch]
                    set_visible: self.request.is_some(),

                    gtk::Box {
                        set_spacing: 6,

                        gtk::Label {
                            set_label: &t!("see_all"),
                        },

                        gtk::Image {
                            set_icon_name: Some("right"),
                        }
                    }
                }
            },

            gtk::Box {
                set_expand: true,
                set_height_request: self.size,

                gtk::Overlay {
                    #[watch]
                    set_visible: self.visible,

                    add_controller = gtk::EventControllerMotion {
                        connect_enter[sender] => move |_event, _x, _y| {
                            sender.input(CatalogRowInput::Hover(true));
                        },
                        connect_leave => CatalogRowInput::Hover(false),
                    },

                    add_overlay = &gtk::Revealer {
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        set_overflow: gtk::Overflow::Visible,
                        set_margin_start: 12,
                        #[watch]
                        set_reveal_child: self.hover && (self.scroll_posiiton == ScrollPosition::End || self.scroll_posiiton == ScrollPosition::Middle),

                        self.left_button.widget(),
                    },

                    add_overlay = &gtk::Revealer {
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        set_overflow: gtk::Overflow::Visible,
                        set_margin_end: 12,
                        #[watch]
                        set_reveal_child: self.hover && (self.scroll_posiiton == ScrollPosition::Start || self.scroll_posiiton == ScrollPosition::Middle),

                        self.right_button.widget(),
                    },

                    #[local_ref]
                    scrolled_window -> gtk::ScrolledWindow {
                        set_expand: true,
                        set_propagate_natural_height: true,
                        set_vscrollbar_policy: gtk::PolicyType::Never,
                        set_hscrollbar_policy: gtk::PolicyType::External,

                        #[wrap(Some)]
                        set_hadjustment = &gtk::Adjustment {
                            connect_page_increment_notify => CatalogRowInput::ScrollLayout,
                            connect_value_changed => CatalogRowInput::ScrollLayout,
                        },

                        #[local_ref]
                        items -> gtk::Box {
                            set_halign: gtk::Align::Start,
                            set_expand: true,
                            set_spacing: 16,
                            set_margin_horizontal: 16,
                        },
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let settings = gio::Settings::new(APP_ID);

        let size = match &init.r#type {
            Some(r#type) if r#type == "channel" || r#type == "radio" => ITEM_MIN_SIZE,
            _ => ITEM_MAX_SIZE,
        };

        let icon = Image::builder()
            .launch(
                ImageInit::builder()
                    .source(init.icon)
                    .size(CATALOG_ICON_SIZE)
                    .placeholder("play")
                    .placeholder_size(CATALOG_ICON_SIZE)
                    .content_fit(gtk::ContentFit::Cover)
                    .build(),
            )
            .detach();

        let scrolled_window = layout::nested_scrolled_window();

        let left_button = Button::builder()
            .launch(ButtonInit { icon: "left" })
            .forward(sender.input_sender(), |msg| match msg {
                ButtonOutput::Click => CatalogRowInput::ScrollLeft,
            });

        let right_button = Button::builder()
            .launch(ButtonInit { icon: "right" })
            .forward(sender.input_sender(), |msg| match msg {
                ButtonOutput::Click => CatalogRowInput::ScrollRight,
            });

        let mut items = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        items.extend(init.items);

        Self {
            settings,
            index: 0,
            hover: false,
            visible: true,
            size,
            icon,
            request: init.request,
            r#type: init.r#type,
            addon_name: init.addon_name,
            name: init.name,
            scroll_posiiton: ScrollPosition::Start,
            scrolled_window,
            left_button,
            right_button,
            items,
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        self.index = index.current_index();

        let scrolled_window = &self.scrolled_window;
        let items = self.items.widget();

        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            CatalogRowInput::ScrollLayout => {
                let position = layout::horizontal_scroll_position(&self.scrolled_window);
                self.scroll_posiiton = position;

                let in_view_items = layout::in_view(
                    &self.items,
                    &self.scrolled_window,
                    gtk::Orientation::Horizontal,
                );

                for index in 0..self.items.len() {
                    if in_view_items.contains(&index) {
                        self.items.guard().send(index, ItemBoxInput::LoadImage);
                        self.items.guard().send(index, ItemBoxInput::Show);
                    } else {
                        self.items.guard().send(index, ItemBoxInput::Hide);
                    }
                }
            }
            CatalogRowInput::Show => {
                self.visible = true;
            }
            CatalogRowInput::Hide => {
                self.visible = false;
            }
            CatalogRowInput::Hover(state) => {
                self.hover = state;
            }
            CatalogRowInput::ScrollLeft => {
                self.scrolled_window
                    .emit_by_name::<bool>("scroll-child", &[&gtk::ScrollType::StepBackward, &true]);
            }
            CatalogRowInput::ScrollRight => {
                self.scrolled_window
                    .emit_by_name::<bool>("scroll-child", &[&gtk::ScrollType::StepForward, &true]);
            }
            CatalogRowInput::ShowAllClicked => {
                APP_BROKER.send(AppMsg::OpenDiscover(self.request.clone()))
            }
        }
    }
}
