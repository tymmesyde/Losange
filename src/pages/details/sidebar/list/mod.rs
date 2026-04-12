pub mod list_item;

use std::{fmt::Debug, time::Duration};

use gtk::prelude::*;
use list_item::ListItem;
use relm4::{
    component,
    gtk::{self, glib, NoSelection},
    typed_view::list::TypedListView,
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};

use crate::components::spinner::Spinner;

#[derive(Debug)]
pub enum ListInput {
    Update(Vec<ListItem>),
    UpdateItems(Vec<ListItem>),
    Clear,
}

#[derive(Debug)]
pub enum ListOutput {
    Clicked(usize),
}

pub struct List {
    scrolled_window: gtk::ScrolledWindow,
    items: TypedListView<ListItem, NoSelection>,
    loading: bool,
    debouncer: Option<glib::JoinHandle<()>>,
}

#[component(pub)]
impl SimpleComponent for List {
    type Input = ListInput;
    type Output = ListOutput;
    type Init = ();

    view! {
        gtk::Box {
            set_expand: true,

            #[transition = "Crossfade"]
            if model.loading {
                #[template]
                Spinner {}
            } else {
                gtk::Box {
                    #[local_ref]
                    scrolled_window -> gtk::ScrolledWindow {
                        set_expand: true,

                        #[local_ref]
                        items -> gtk::ListView {
                            add_css_class: relm4::css::classes::BOXED_LIST,
                            set_valign: gtk::Align::Start,
                            set_expand: true,
                            set_single_click_activate: true,

                            connect_activate[sender] => move |_, index| {
                                sender.output_sender().emit(ListOutput::Clicked(index as usize));
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let scrolled_window = gtk::ScrolledWindow::new();

        let items = TypedListView::<ListItem, NoSelection>::new();

        let model = Self {
            scrolled_window,
            items,
            loading: true,
            debouncer: None,
        };

        let scrolled_window = &model.scrolled_window;
        let items = &model.items.view;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ListInput::Update(items) => {
                if let Some(debouncer) = self.debouncer.take() {
                    debouncer.abort();
                }

                self.debouncer = Some(relm4::spawn_local(async move {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    sender.input_sender().emit(ListInput::UpdateItems(items));
                }));
            }
            ListInput::UpdateItems(items) => {
                self.items.clear();
                for item in items {
                    self.items.append(item);
                }

                self.loading = false;
            }
            ListInput::Clear => {
                self.items.clear();
                self.loading = true;
            }
        }
    }
}
