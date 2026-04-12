use adw::prelude::*;
use itertools::Itertools;
use relm4::{
    adw, css, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::models::{self, library::LIBRARY_STATE};

use crate::{
    app::AppMsg,
    components::{
        dropdown::{DropDown, DropDownInput, DropDownOutput},
        meta_item::grid::{GridMetaItem, GridMetaItemInput, GridMetaItemOutput},
    },
    APP_BROKER,
};

#[derive(Debug)]
pub enum LibraryPageInput {
    Load,
    Update,
    LoadMore,
    TypeChanged(usize),
    OrderChanged(usize),
    ItemClicked(String),
}

pub struct LibraryPage {
    types: Controller<DropDown>,
    orders: Controller<DropDown>,
    grid: Controller<GridMetaItem>,
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
            if library.items.is_empty() {
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

                    model.grid.widget(),
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let library = LIBRARY_STATE.read_inner();

        LIBRARY_STATE.subscribe(sender.input_sender(), |_| LibraryPageInput::Update);

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

        let grid =
            GridMetaItem::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GridMetaItemOutput::Clicked(id) => LibraryPageInput::ItemClicked(id),
                    GridMetaItemOutput::ScrolledToBottom => LibraryPageInput::LoadMore,
                });

        let model = LibraryPage {
            types,
            orders,
            grid,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let library = LIBRARY_STATE.read_inner();
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

                self.grid
                    .emit(GridMetaItemInput::Update(state.items.clone()));
            }
            LibraryPageInput::LoadMore => {
                models::library::load_next_items();
            }
            LibraryPageInput::TypeChanged(index) => {
                models::library::load_with_type(index);
            }
            LibraryPageInput::OrderChanged(index) => {
                models::library::load_with_order(index);
            }
            LibraryPageInput::ItemClicked(id) => {
                let state = LIBRARY_STATE.read_inner();

                if let Some(item) = state.items.iter().find(|item| item.id == id) {
                    APP_BROKER.send(AppMsg::OpenDetails((
                        item.id.to_owned(),
                        item.r#type.to_owned(),
                    )))
                }
            }
        }
    }
}
