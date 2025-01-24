use gtk::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::{gtk, RelmWidgetExt};
use rust_i18n::t;

pub struct MenuOptionInit {
    pub id: i32,
    pub label: Option<String>,
    pub active: bool,
    pub group: gtk::CheckButton,
}

#[derive(Debug)]
pub enum MenuOptionInput {
    Toggled,
}

#[derive(Debug)]
pub enum MenuOptionOutput {
    Clicked(i32),
}

pub struct MenuOption {
    pub id: i32,
    label: Option<String>,
    active: bool,
    group: gtk::CheckButton,
}

#[relm4::factory(pub)]
impl FactoryComponent for MenuOption {
    type Init = MenuOptionInit;
    type Input = MenuOptionInput;
    type Output = MenuOptionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::ListBoxRow {
            gtk::CheckButton {
                set_margin_all: 4,
                set_label: Some(self.label.as_ref().map_or(&t!("original"), |label| label)),
                set_active: self.active,
                set_group: Some(&self.group),
                connect_toggled => MenuOptionInput::Toggled,
            }
        }
    }

    fn init_model(init: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self {
            id: init.id,
            label: init.label,
            active: init.active,
            group: init.group,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            MenuOptionInput::Toggled => {
                if !self.active {
                    sender
                        .output_sender()
                        .emit(MenuOptionOutput::Clicked(self.id));
                }
            }
        }
    }
}
