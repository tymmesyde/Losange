use adw::prelude::*;
use relm4::{
    adw, factory::FactoryVecDeque, gtk, ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent,
};
use stremio_core_losange::models::{self, home::HOME_STATE};

use crate::{
    common::layout,
    components::catalog_row::{CatalogRow, CatalogRowInput},
};

#[derive(Debug)]
pub enum HomePageInput {
    Load,
    Update,
    LayoutUpdate,
}

pub struct HomePage {
    scrolled_window: gtk::ScrolledWindow,
    catalogs: FactoryVecDeque<CatalogRow>,
}

#[relm4::component(pub)]
impl SimpleComponent for HomePage {
    type Init = ();
    type Input = HomePageInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "HomePage",
            set_tag: Some("home"),
            connect_realize => HomePageInput::Load,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[local_ref]
                scrolled_window -> gtk::ScrolledWindow {
                    set_expand: true,

                    #[wrap(Some)]
                    set_vadjustment = &gtk::Adjustment {
                        connect_page_increment_notify => HomePageInput::LayoutUpdate,
                        connect_value_changed => HomePageInput::LayoutUpdate,
                    },

                    #[local_ref]
                    catalogs -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Start,
                        set_margin_bottom: 12,
                        set_spacing: 24,
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
        HOME_STATE.subscribe(sender.input_sender(), |_| HomePageInput::Update);

        let scrolled_window = gtk::ScrolledWindow::new();

        let catalogs = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = HomePage {
            scrolled_window,
            catalogs,
        };

        let scrolled_window = &model.scrolled_window;
        let catalogs = model.catalogs.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            HomePageInput::Load => {
                models::home::load();
                models::ctx::sync_with_api();
            }
            HomePageInput::Update => {
                let state = HOME_STATE.read_inner();

                for (i, catalog) in state.catalogs.iter().enumerate() {
                    if let Some(catalog_row) = self.catalogs.get(i) {
                        if catalog.items.len() != catalog_row.items.len()
                            || catalog.items.iter().zip(catalog_row.items.iter()).any(
                                |(state_item, row_item)| {
                                    state_item.id != row_item.id
                                        || state_item.progress != row_item.progress
                                        || state_item.new_videos != row_item.new_videos
                                        || state_item.last_stream != row_item.last_stream
                                },
                            )
                        {
                            self.catalogs.guard().remove(i);
                            self.catalogs.guard().insert(i, catalog.to_owned());
                        }
                    } else {
                        self.catalogs.guard().insert(i, catalog.to_owned());
                    }
                }

                while self.catalogs.len() > state.catalogs.len() {
                    self.catalogs.guard().pop_back();
                }
            }
            HomePageInput::LayoutUpdate => {
                let in_view_catalogs = layout::in_view(
                    &self.catalogs,
                    &self.scrolled_window,
                    gtk::Orientation::Vertical,
                );

                for index in 0..self.catalogs.len() {
                    if in_view_catalogs.contains(&index) {
                        self.catalogs.guard().send(index, CatalogRowInput::Show);
                    } else {
                        self.catalogs.guard().send(index, CatalogRowInput::Hide);
                    }
                }

                if let (Some(first), Some(last)) =
                    (in_view_catalogs.first(), in_view_catalogs.last())
                {
                    models::home::load_catalog(*first, *last);
                }
            }
        }
    }
}
