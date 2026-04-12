use adw::prelude::*;
use itertools::Itertools;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::{
    models::{self, discover::DISCOVER_STATE},
    stremio_core::types::addon::ResourceRequest,
};

use crate::{
    app::AppMsg,
    common::translate,
    components::{
        dropdown::{DropDown, DropDownInput, DropDownOutput},
        meta_item::grid::{GridMetaItem, GridMetaItemInput, GridMetaItemOutput},
    },
    APP_BROKER,
};

#[derive(Debug)]
pub enum DiscoverPageInput {
    Load(Option<ResourceRequest>),
    Update,
    LoadMore,
    TypeChanged(usize),
    CatalogChanged(usize),
    GenreChanged(usize),
    ItemClicked(String),
}

pub struct DiscoverPage {
    types: Controller<DropDown>,
    catalogs: Controller<DropDown>,
    genres: Controller<DropDown>,
    grid: Controller<GridMetaItem>,
}

#[relm4::component(pub)]
impl SimpleComponent for DiscoverPage {
    type Init = ();
    type Input = DiscoverPageInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "Discover",
            set_tag: Some("discover"),
            connect_realize => DiscoverPageInput::Load(None),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_margin_horizontal: 12,
                    set_spacing: 6,

                    model.types.widget(),
                    model.catalogs.widget(),
                    model.genres.widget(),
                },

                model.grid.widget(),
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        DISCOVER_STATE.subscribe(sender.input_sender(), |_| DiscoverPageInput::Update);

        let types =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => DiscoverPageInput::TypeChanged(index),
                });

        let catalogs =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => DiscoverPageInput::CatalogChanged(index),
                });

        let genres =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => DiscoverPageInput::GenreChanged(index),
                });

        let grid =
            GridMetaItem::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GridMetaItemOutput::Clicked(id) => DiscoverPageInput::ItemClicked(id),
                    GridMetaItemOutput::ScrolledToBottom => DiscoverPageInput::LoadMore,
                });

        let model = DiscoverPage {
            types,
            catalogs,
            genres,
            grid,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            DiscoverPageInput::Load(request) => {
                models::discover::load(request);
            }
            DiscoverPageInput::Update => {
                let state = DISCOVER_STATE.read_inner();

                let types = state
                    .types
                    .iter()
                    .map(|selectable| t!(selectable.r#type.as_str()).to_string())
                    .collect_vec();

                self.types.emit(DropDownInput::Update(types));

                let selected_type = state
                    .types
                    .iter()
                    .position(|selectable| selectable.selected);

                if let Some(index) = selected_type {
                    self.types.emit(DropDownInput::Select(index));
                }

                let catalogs = state
                    .catalogs
                    .iter()
                    .map(|selectable| selectable.catalog.to_owned())
                    .collect_vec();

                self.catalogs.emit(DropDownInput::Update(catalogs));

                let selected_catalog = state
                    .catalogs
                    .iter()
                    .position(|selectable| selectable.selected);

                if let Some(index) = selected_catalog {
                    self.catalogs.emit(DropDownInput::Select(index));
                }

                let genres = state
                    .genres
                    .iter()
                    .map(|selectable| translate::genre(&selectable.value))
                    .collect_vec();

                self.genres.emit(DropDownInput::Update(genres));

                self.grid
                    .emit(GridMetaItemInput::Update(state.items.clone()));
            }
            DiscoverPageInput::LoadMore => {
                models::discover::load_next_items();
            }
            DiscoverPageInput::TypeChanged(index) => {
                models::discover::load_with_type(index);
            }
            DiscoverPageInput::CatalogChanged(index) => {
                models::discover::load_with_catalog(index);
            }
            DiscoverPageInput::GenreChanged(index) => {
                models::discover::load_with_genre(index);
            }
            DiscoverPageInput::ItemClicked(id) => {
                let state = DISCOVER_STATE.read_inner();

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
