use relm4::{
    gtk::{
        self,
        prelude::{AdjustmentExt, WidgetExt},
        NoSelection,
    },
    typed_view::grid::TypedGridView,
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};
use stremio_core_losange::types::item::Item;

use crate::{
    common::layout,
    components::{meta_item::item::MetaItem, spinner::Spinner},
};

#[derive(Debug)]
pub enum GridMetaItemInput {
    Update(Vec<Item>),
    Scrolled,
    Clicked(u32),
}

#[derive(Debug)]
pub enum GridMetaItemOutput {
    Clicked(String),
    ScrolledToBottom,
}

pub struct GridMetaItem {
    scrolled_window: gtk::ScrolledWindow,
    items: TypedGridView<MetaItem, NoSelection>,
}

#[relm4::component(pub)]
impl SimpleComponent for GridMetaItem {
    type Init = ();
    type Input = GridMetaItemInput;
    type Output = GridMetaItemOutput;

    view! {
        gtk::Box {
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
                    connect_changed => GridMetaItemInput::Scrolled,
                    connect_value_changed => GridMetaItemInput::Scrolled,
                },

                #[local_ref]
                items -> gtk::GridView {
                    set_valign: gtk::Align::Start,
                    set_halign: gtk::Align::Fill,
                    set_margin_horizontal: 6,
                    set_max_columns: 25,
                    set_single_click_activate: true,

                    connect_activate[sender] => move |_, index| {
                        sender.input_sender().emit(GridMetaItemInput::Clicked(index));
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
        let scrolled_window = gtk::ScrolledWindow::new();
        let items = TypedGridView::<MetaItem, NoSelection>::new();

        let model = GridMetaItem {
            scrolled_window,
            items,
        };

        let scrolled_window = &model.scrolled_window;
        let items = &model.items.view;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            GridMetaItemInput::Update(items) => {
                for (i, item) in items.iter().enumerate() {
                    if i >= self.items.len() as usize {
                        self.items.append(item.into());
                    } else if let Some(current_item) = self.items.get(i as u32) {
                        let current_item = current_item.borrow();
                        if items[i].id != current_item.id {
                            self.items.insert(i as u32, item.into());
                        }
                    }
                }

                while self.items.len() > items.len() as u32 {
                    self.items.remove(self.items.len() - 1);
                }
            }
            GridMetaItemInput::Scrolled => {
                if layout::scrolled_to_bottom(&self.scrolled_window) {
                    sender
                        .output_sender()
                        .emit(GridMetaItemOutput::ScrolledToBottom);
                }
            }
            GridMetaItemInput::Clicked(index) => {
                if let Some(item) = self.items.get(index) {
                    let id = item.borrow().id.clone();
                    sender.output_sender().emit(GridMetaItemOutput::Clicked(id));
                }
            }
        }
    }
}
