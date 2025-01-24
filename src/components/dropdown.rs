use adw::prelude::*;
use itertools::Itertools;
use relm4::{adw, gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
pub enum DropDownInput {
    Update(Vec<String>),
    Selected(usize),
}

#[derive(Debug)]
pub enum DropDownOutput {
    Selected(usize),
}

pub struct DropDown {
    string_list: gtk::StringList,
    selected: u32,
}

#[relm4::component(pub)]
impl SimpleComponent for DropDown {
    type Init = ();
    type Input = DropDownInput;
    type Output = DropDownOutput;

    view! {
        gtk::DropDown {
            add_css_class: relm4::css::classes::FLAT,

            #[watch]
            set_visible: model.string_list.n_items() > 0,

            #[watch]
            #[block_signal(selected_handler)]
            set_model: Some(&model.string_list),

            #[watch]
            #[block_signal(selected_handler)]
            set_selected: model.selected,

            connect_selected_item_notify[sender] => move |dropdown| {
                let index = dropdown.selected() as usize;
                sender.input_sender().emit(DropDownInput::Selected(index));
            } @selected_handler,
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let string_list = gtk::StringList::new(&[]);

        let model = Self {
            string_list,
            selected: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            DropDownInput::Update(list) => {
                self.string_list = gtk::StringList::new(
                    list.iter()
                        .map(|label| label.as_str())
                        .collect_vec()
                        .as_slice(),
                );
            }
            DropDownInput::Selected(index) => {
                self.selected = index as u32;
                sender.output_sender().emit(DropDownOutput::Selected(index));
            }
        }
    }
}
