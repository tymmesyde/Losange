use std::fmt::Debug;

use gtk::prelude::*;
use relm4::{
    component, gtk,
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};

use crate::components::spinner::Spinner;

#[derive(Debug)]
pub enum ListItemOutput {
    Clicked(usize),
}

#[derive(Debug)]
pub enum ListInput<I> {
    Update(Vec<I>),
    Clear,
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
        Input = (),
        Output = ListItemOutput,
    >,
    C::Init: Debug + Clone,
{
    scrolled_window: gtk::ScrolledWindow,
    items: FactoryVecDeque<C>,
    loading: bool,
}

#[component(pub)]
impl<C> SimpleComponent for List<C>
where
    C: FactoryComponent<
        Index = DynamicIndex,
        ParentWidget = gtk::ListBox,
        Input = (),
        Output = ListItemOutput,
    >,
    C::Init: Debug + Clone,
{
    type Input = ListInput<C::Init>;
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
                        items -> gtk::ListBox {
                            add_css_class: relm4::css::classes::BOXED_LIST,
                            set_valign: gtk::Align::Start,
                            set_expand: true,
                            set_selection_mode: gtk::SelectionMode::None,
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

        let items = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.output_sender(), |msg| match msg {
                ListItemOutput::Clicked(index) => ListOutput::Clicked(index),
            });

        let model = Self {
            scrolled_window,
            items,
            loading: true,
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
                self.items.extend(items);
                self.loading = false;
            }
            ListInput::Clear => {
                self.items.guard().clear();
                self.loading = true;
            }
        }
    }
}
