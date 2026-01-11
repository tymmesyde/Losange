use adw::prelude::*;
use itertools::Itertools;
use relm4::{
    adw, factory::FactoryVecDeque, gtk, Component, ComponentController, ComponentParts,
    ComponentSender, Controller, RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::{
    models::{self, discover::DISCOVER_STATE},
    stremio_core::types::addon::ResourceRequest,
};

use crate::{
    common::{layout, translate},
    components::{
        dropdown::{DropDown, DropDownInput, DropDownOutput},
        item_box::{ItemBox, ItemBoxInput},
        spinner::Spinner,
    },
};

#[derive(Debug)]
pub enum DiscoverPageInput {
    Load(Option<ResourceRequest>),
    Update,
    LayoutUpdate,
    TypeChanged(usize),
    CatalogChanged(usize),
    GenreChanged(usize),
}

pub struct DiscoverPage {
    types: Controller<DropDown>,
    catalogs: Controller<DropDown>,
    genres: Controller<DropDown>,
    scrolled_window: gtk::ScrolledWindow,
    items: FactoryVecDeque<ItemBox<gtk::FlowBox>>,
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

                #[template]
                Spinner {
                    #[watch]
                    set_visible: model.items.is_empty(),
                },

                #[local_ref]
                scrolled_window -> gtk::ScrolledWindow {
                    set_expand: true,

                    #[watch]
                    set_visible: !model.items.is_empty(),

                    #[wrap(Some)]
                    set_vadjustment = &gtk::Adjustment {
                        connect_changed => DiscoverPageInput::LayoutUpdate,
                        connect_value_changed => DiscoverPageInput::LayoutUpdate,
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

                        connect_map => DiscoverPageInput::LayoutUpdate,
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
        DISCOVER_STATE.subscribe(sender.input_sender(), |_| DiscoverPageInput::Update);

        let scrolled_window = gtk::ScrolledWindow::new();

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

        let items = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let model = DiscoverPage {
            types,
            catalogs,
            genres,
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
            DiscoverPageInput::LayoutUpdate => {
                self.update_items();

                if layout::scrolled_to_bottom(&self.scrolled_window) {
                    models::discover::load_next_items();
                }
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
        }
    }
}

impl DiscoverPage {
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
