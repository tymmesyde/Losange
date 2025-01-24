use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
pub enum ButtonInput {
    Pressed,
    Released,
    Leave,
}

#[derive(Debug)]
pub enum ButtonOutput {
    Click,
}

pub struct ButtonInit {
    pub icon: &'static str,
}

pub struct Button {
    pressed_timeout: Option<glib::SourceId>,
}

#[relm4::component(pub)]
impl SimpleComponent for Button {
    type Input = ButtonInput;
    type Output = ButtonOutput;
    type Init = ButtonInit;

    view! {
        gtk::Box {
            add_css_class: "catalog-button",
            set_size_request: (34, 34),
            set_focusable: true,

            add_controller = gtk::GestureClick {
                connect_pressed[sender] => move |_, _, _, _| {
                    sender.input_sender().emit(ButtonInput::Pressed);
                },
                connect_released[sender] => move |_, _, _, _| {
                    sender.input_sender().emit(ButtonInput::Released);
                }
            },

            add_controller = gtk::EventControllerMotion {
                connect_leave[sender] => move |_| {
                    sender.input_sender().emit(ButtonInput::Leave);
                },
            },

            gtk::Image {
                set_expand: true,
                set_icon_name: Some(init.icon),
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Button {
            pressed_timeout: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ButtonInput::Pressed => {
                self.remove_pressed_timeout();
                self.add_pressed_timeout(sender);
            }
            ButtonInput::Released => {
                sender.output_sender().emit(ButtonOutput::Click);
                self.remove_pressed_timeout();
            }
            ButtonInput::Leave => {
                self.remove_pressed_timeout();
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        self.remove_pressed_timeout();
    }
}

impl Button {
    fn add_pressed_timeout(&mut self, sender: ComponentSender<Self>) {
        self.pressed_timeout = Some(glib::timeout_add_local(
            Duration::from_millis(150),
            move || {
                sender.output_sender().emit(ButtonOutput::Click);

                glib::ControlFlow::Continue
            },
        ))
    }

    fn remove_pressed_timeout(&mut self) {
        if let Some(pressed_timeout) = self.pressed_timeout.take() {
            pressed_timeout.remove();
        }
    }
}
