use adw::prelude::*;
use relm4::{
    adw, css, factory::FactoryVecDeque, gtk, ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::models::{self, search::SEARCH_STATE};

use crate::components::{catalog_row::CatalogRow, spinner::Spinner};

#[derive(Debug)]
pub enum SearchPageInput {
    Load(String),
    Update,
    Showing,
}

pub struct SearchPage {
    entry: gtk::SearchEntry,
    catalogs: FactoryVecDeque<CatalogRow>,
}

#[relm4::component(pub)]
impl SimpleComponent for SearchPage {
    type Init = ();
    type Input = SearchPageInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "Search",
            set_tag: Some("search"),
            connect_showing => SearchPageInput::Showing,

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {

                    #[wrap(Some)]
                    set_title_widget = &gtk::Box {

                        #[local_ref]
                        entry -> gtk::SearchEntry {
                            set_width_request: 300,
                            set_placeholder_text: Some(&t!("search")),

                            connect_changed[sender] => move |entry| {
                                let value = entry.text().to_string();
                                sender.input(SearchPageInput::Load(value));
                            },
                        }
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::Box {
                    if state.loading {
                        #[template]
                        Spinner {}
                    } else if model.catalogs.is_empty() {
                        adw::StatusPage {
                            add_css_class: css::classes::COMPACT,
                            set_title: &t!("search_status_title"),
                            set_description: Some(&t!("search_status_description")),
                        }
                    } else {
                        gtk::ScrolledWindow {
                            set_expand: true,

                            #[local_ref]
                            catalogs -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_valign: gtk::Align::Start,
                                set_margin_bottom: 24,
                                set_spacing: 24,
                            }
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
        let state = SEARCH_STATE.read_inner();

        SEARCH_STATE.subscribe(sender.input_sender(), |_| SearchPageInput::Update);

        let entry = gtk::SearchEntry::new();

        let catalogs = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = SearchPage { entry, catalogs };

        let entry = &model.entry;
        let catalogs = model.catalogs.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let state = SEARCH_STATE.read_inner();
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SearchPageInput::Load(query) => {
                if query.is_empty() {
                    models::search::unload();
                } else {
                    if query != self.entry.text() {
                        self.entry.set_text(&query);
                    }

                    models::search::load(query);
                    models::search::load_catalog(0, 100);
                }
            }
            SearchPageInput::Update => {
                let state = SEARCH_STATE.read_inner();

                if !state.catalogs.is_empty() {
                    for (i, catalog) in state.catalogs.iter().enumerate() {
                        if let Some(catalog_row) = self.catalogs.get(i) {
                            if catalog.items.len() != catalog_row.items.len() {
                                self.catalogs.guard().remove(i);
                                self.catalogs.guard().insert(i, catalog.to_owned());
                            }
                        } else {
                            self.catalogs.guard().push_back(catalog.to_owned());
                        }
                    }

                    for i in state.catalogs.len()..self.catalogs.len() {
                        self.catalogs.guard().remove(i);
                    }
                } else {
                    self.catalogs.guard().clear();
                }
            }
            SearchPageInput::Showing => {
                self.entry.grab_focus();
            }
        }
    }
}
