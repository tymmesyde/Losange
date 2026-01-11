use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use relm4::factory::{FactoryVecDeque, FactoryView};
use relm4::{css, prelude::*};
use relm4::{gtk, FactorySender};
use rust_i18n::t;
use stremio_core_losange::stremio_core::types::addon::ResourceRequest;
use stremio_core_losange::types::catalog::Catalog;
use stremio_core_losange::types::item::Item;

use crate::app::AppMsg;
use crate::components::image::{init::ImageInit, Image};
use crate::components::item_box::{ItemBox, ItemBoxInput};
use crate::constants::{APP_ID, CATALOG_ICON_SIZE, ITEM_MAX_SIZE, ITEM_MIN_SIZE};
use crate::APP_BROKER;

pub type CatalogRowInit = Catalog;

#[derive(Debug, Clone)]
pub enum CatalogRowInput {
    UpdateLayout,
    Show,
    Hide,
    ShowAllClicked,
}

pub struct CatalogRow {
    settings: gio::Settings,
    index: usize,
    visible: bool,
    homogeneous: bool,
    size: i32,
    request: Option<ResourceRequest>,
    icon: AsyncController<Image>,
    r#type: Option<String>,
    addon_name: Option<String>,
    name: Option<String>,
    pub items: Vec<Item>,
    items_list_width: Rc<Cell<i32>>,
    pub items_list: FactoryVecDeque<ItemBox<gtk::Box>>,
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
                set_height_request: self.size,

                #[local_ref]
                items_list -> gtk::Box {
                    set_valign: gtk::Align::Start,
                    set_halign: gtk::Align::Fill,
                    set_expand: true,
                    set_spacing: 16,
                    set_margin_horizontal: 16,
                    set_overflow: gtk::Overflow::Hidden,

                    #[watch]
                    set_visible: self.visible,

                    #[watch]
                    set_homogeneous: self.homogeneous,
                },
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
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

        let mut items_list = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        if let Some(item) = init.items.first() {
            items_list.guard().push_front(item.clone());
            items_list.broadcast(ItemBoxInput::LoadImage);
            items_list.broadcast(ItemBoxInput::Show);
        }

        Self {
            settings,
            index: 0,
            visible: true,
            homogeneous: true,
            size,
            request: init.request,
            icon,
            r#type: init.r#type,
            addon_name: init.addon_name,
            name: init.name,
            items: init.items,
            items_list_width: Rc::new(Cell::new(0)),
            items_list,
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

        let items_list = self.items_list.widget();

        let widgets = view_output!();

        widgets.items_list.add_tick_callback({
            let sender = sender.clone();
            let items_list_width = self.items_list_width.clone();

            move |widget, _clock| {
                let width = widget.width();
                let prev_width = items_list_width.get();

                if width != prev_width && width > 0 {
                    items_list_width.set(width);
                    sender.input(CatalogRowInput::UpdateLayout);
                }

                glib::ControlFlow::Continue
            }
        });

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            CatalogRowInput::UpdateLayout => {
                let widget = self.items_list.widget();

                let width = widget.width();
                let margin = widget.margin_start() + widget.margin_end();
                let spacing = widget.spacing();

                if let Some(item_widget) = widget.first_child() {
                    let item_width = item_widget.width();

                    let available_width = width - margin;
                    let visible_items =
                        ((available_width + spacing) / (item_width + spacing)).max(1) as usize;

                    if visible_items != self.items_list.len() {
                        let mut items_list = self.items_list.guard();
                        let items = self.items.iter().take(visible_items);

                        for (i, item) in items.enumerate() {
                            if let Some(list_item) = items_list.get(i) {
                                if list_item.id != item.id {
                                    items_list.remove(i);
                                    items_list.insert(i, item.to_owned());
                                    items_list.send(i, ItemBoxInput::LoadImage);
                                    items_list.send(i, ItemBoxInput::Show);
                                }
                            } else {
                                items_list.insert(i, item.to_owned());
                                items_list.send(i, ItemBoxInput::LoadImage);
                                items_list.send(i, ItemBoxInput::Show);
                            }
                        }

                        while items_list.len() > visible_items {
                            items_list.pop_back();
                        }

                        if items_list.len() < visible_items {
                            self.homogeneous = false;
                        }
                    }
                }
            }
            CatalogRowInput::Show => {
                self.visible = true;
            }
            CatalogRowInput::Hide => {
                self.visible = false;
            }
            CatalogRowInput::ShowAllClicked => {
                APP_BROKER.send(AppMsg::OpenDiscover(self.request.clone()));
            }
        }
    }
}
