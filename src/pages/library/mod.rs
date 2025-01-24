use adw::prelude::*;
use itertools::Itertools;
use relm4::{
    adw, css, factory::FactoryVecDeque, gtk, Component, ComponentController, ComponentParts,
    ComponentSender, Controller, RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::models::{self, library::LIBRARY_STATE};

use crate::{
    app::AppMsg,
    common::layout,
    components::{
        dropdown::{DropDown, DropDownInput, DropDownOutput},
        item_box::{ItemBox, ItemBoxInput},
    },
    APP_BROKER,
};

#[derive(Debug)]
pub enum LibraryPageInput {
    Load,
    Update,
    LayoutUpdate,
    TypeChanged(usize),
    OrderChanged(usize),
}

pub struct LibraryPage {
    types: Controller<DropDown>,
    orders: Controller<DropDown>,
    scrolled_window: gtk::ScrolledWindow,
    items: FactoryVecDeque<ItemBox<gtk::FlowBox>>,
}

#[relm4::component(pub)]
impl SimpleComponent for LibraryPage {
    type Init = ();
    type Input = LibraryPageInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "Library",
            set_tag: Some("library"),
            connect_realize => LibraryPageInput::Load,

            #[transition = "Crossfade"]
            if model.items.is_empty() {
                adw::StatusPage {
                    add_css_class: css::classes::COMPACT,
                    set_title: &t!("library_empty_title"),
                    set_description: Some(&t!("library_empty_description")),

                    gtk::Button {
                        set_halign: gtk::Align::Center,
                        set_css_classes: &[css::classes::PILL, css::classes::SUGGESTED_ACTION],
                        set_label: &t!("library_empty_button"),
                        connect_clicked => |_| {
                            APP_BROKER.send(AppMsg::OpenHome);
                        },
                    }
                }
            } else {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_margin_horizontal: 12,
                        set_spacing: 6,

                        model.types.widget(),
                        model.orders.widget(),
                    },

                    #[local_ref]
                    scrolled_window -> gtk::ScrolledWindow {
                        set_expand: true,

                        #[wrap(Some)]
                        set_vadjustment = &gtk::Adjustment {
                            connect_changed => LibraryPageInput::LayoutUpdate,
                            connect_value_changed => LibraryPageInput::LayoutUpdate,
                        },

                        #[local_ref]
                        items -> gtk::FlowBox {
                            set_valign: gtk::Align::Start,
                            set_halign: gtk::Align::Fill,
                            set_row_spacing: 12,
                            set_column_spacing: 12,
                            set_margin_horizontal: 12,
                            set_margin_top: 6,
                            set_margin_bottom: 12,
                            set_homogeneous: true,
                            set_max_children_per_line: 25,
                            set_selection_mode: gtk::SelectionMode::None,

                            connect_map => LibraryPageInput::LayoutUpdate,
                        }
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        LIBRARY_STATE.subscribe(sender.input_sender(), |_| LibraryPageInput::Update);

        let scrolled_window = gtk::ScrolledWindow::new();

        let types =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => LibraryPageInput::TypeChanged(index),
                });

        let orders =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => LibraryPageInput::OrderChanged(index),
                });

        let items = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let model = LibraryPage {
            types,
            orders,
            scrolled_window,
            items,
        };

        let scrolled_window = &model.scrolled_window;
        let items = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            LibraryPageInput::Load => {
                models::library::load(None);
            }
            LibraryPageInput::Update => {
                let state = LIBRARY_STATE.read_inner();

                let types = state
                    .types
                    .iter()
                    .map(|selectable| {
                        selectable
                            .r#type
                            .as_ref()
                            .map_or(t!("all"), |name| t!(name.as_str()))
                            .to_string()
                    })
                    .collect_vec();

                self.types.emit(DropDownInput::Update(types));

                let orders = state
                    .orders
                    .iter()
                    .map(|selectable| selectable.sort.to_owned())
                    .map(|name| format!("{:?}", name).to_lowercase())
                    .map(|name| t!(name).to_string())
                    .collect_vec();

                self.orders.emit(DropDownInput::Update(orders));

                for (i, item) in state.items.iter().enumerate() {
                    if i >= self.items.len() {
                        self.items.guard().push_back(item.to_owned());
                    } else if state.items[i].id != self.items[i].id {
                        self.items.guard().insert(i, item.to_owned());
                    }
                }

                while self.items.len() > state.items.len() {
                    self.items.guard().pop_back();
                }

                self.update_items();
            }
            LibraryPageInput::LayoutUpdate => {
                self.update_items();

                if layout::scrolled_to_bottom(&self.scrolled_window) {
                    models::library::load_next_items();
                }
            }
            LibraryPageInput::TypeChanged(index) => {
                models::library::load_with_type(index);
            }
            LibraryPageInput::OrderChanged(index) => {
                models::library::load_with_order(index);
            }
        }
    }
}

impl LibraryPage {
    fn update_items(&mut self) {
        let in_view_items = layout::in_view(
            &self.items,
            &self.scrolled_window,
            gtk::Orientation::Vertical,
        );

        if !in_view_items.is_empty() {
            for index in 0..self.items.len() {
                if in_view_items.contains(&index) {
                    self.items.guard().send(index, ItemBoxInput::LoadImage);
                    self.items.guard().send(index, ItemBoxInput::Show);
                } else {
                    self.items.guard().send(index, ItemBoxInput::Hide);
                }
            }
        }
    }
}
