use gtk::prelude::*;
use relm4::{
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};

pub type TagInit = String;

#[derive(Debug)]
pub enum TagInput {
    Clicked,
}

#[derive(Debug)]
pub enum TagOutput {
    Clicked(String),
}

pub struct Tag {
    label: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for Tag {
    type Init = TagInit;
    type Input = TagInput;
    type Output = TagOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            add_css_class: "medium-tag",
            set_label: &self.label,
            connect_clicked => TagInput::Clicked,
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { label: init }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            TagInput::Clicked => sender
                .output_sender()
                .emit(TagOutput::Clicked(self.label.to_owned())),
        }
    }
}
