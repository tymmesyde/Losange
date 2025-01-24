use std::fmt::Debug;

use gtk::prelude::*;
use relm4::{
    component, gtk,
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};

use crate::common::layout;

#[derive(Debug)]
pub enum ListItemInput {
    Show,
    Hide,
}

#[derive(Debug)]
pub enum ListItemOutput {
    Clicked(usize),
}

#[derive(Debug)]
pub enum ListInput<I> {
    Update(Vec<I>),
    LayoutUpdate,
}

#[derive(Debug)]
pub enum ListOutput {
    Clicked(usize),
}

pub struct List<C>
where
    C: FactoryComponent<
        Index = DynamicIndex,
        ParentWidget = gtk::ListBox,
        Input = ListItemInput,
        Output = ListItemOutput,
    >,
    C::Init: Debug + Clone,
{
    scrolled_window: gtk::ScrolledWindow,
    items: FactoryVecDeque<C>,
}

#[component(pub)]
impl<C> SimpleComponent for List<C>
where
    C: FactoryComponent<
        Index = DynamicIndex,
        ParentWidget = gtk::ListBox,
        Input = ListItemInput,
        Output = ListItemOutput,
    >,
    C::Init: Debug + Clone,
{
    type Input = ListInput<C::Init>;
    type Output = ListOutput;
    type Init = ();

    view! {
        gtk::Box {
            #[local_ref]
            scrolled_window -> gtk::ScrolledWindow {
                set_expand: true,

                #[wrap(Some)]
                set_vadjustment = &gtk::Adjustment {
                    connect_changed => ListInput::LayoutUpdate,
                    connect_value_changed => ListInput::LayoutUpdate,
                },

                #[local_ref]
                items -> gtk::ListBox {
                    add_css_class: relm4::css::classes::BOXED_LIST,
                    set_valign: gtk::Align::Start,
                    set_expand: true,
                    set_selection_mode: gtk::SelectionMode::None,

                    connect_map => ListInput::LayoutUpdate,
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let scrolled_window = gtk::ScrolledWindow::new();

        let items = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.output_sender(), |msg| match msg {
                ListItemOutput::Clicked(index) => ListOutput::Clicked(index),
            });

        let model = Self {
            scrolled_window,
            items,
        };

        let scrolled_window = &model.scrolled_window;
        let items = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ListInput::Update(items) => {
                self.items.guard().clear();
                for item in items {
                    self.items.guard().push_back(item);
                }

                self.update();
            }
            ListInput::LayoutUpdate => self.update(),
        }
    }
}

impl<C> List<C>
where
    C: FactoryComponent<
        Index = DynamicIndex,
        ParentWidget = gtk::ListBox,
        Input = ListItemInput,
        Output = ListItemOutput,
    >,
    C::Init: Debug + Clone,
{
    fn update(&mut self) {
        let in_view_items = layout::in_view(
            &self.items,
            &self.scrolled_window,
            gtk::Orientation::Vertical,
        );

        if !in_view_items.is_empty() {
            for index in 0..self.items.len() {
                if in_view_items.contains(&index) {
                    self.items.guard().send(index, ListItemInput::Show);
                } else {
                    self.items.guard().send(index, ListItemInput::Hide);
                }
            }
        }
    }
}
