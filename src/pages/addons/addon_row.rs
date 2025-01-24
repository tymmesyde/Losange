use crate::{
    common::format::Format,
    components::image::{init::ImageInit, Image},
};
use adw::prelude::*;
use relm4::{
    adw,
    factory::{DynamicIndex, FactoryComponent},
    gtk,
    prelude::{AsyncComponent, AsyncComponentController, AsyncController},
    FactorySender, RelmWidgetExt,
};
use stremio_core_losange::types::addon::Addon;

#[derive(Debug)]
pub enum AddonRowOutput {
    Clicked(usize),
}

pub struct AddonRow {
    pub icon: AsyncController<Image>,
    pub title: String,
    pub description: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for AddonRow {
    type Input = ();
    type Output = AddonRowOutput;
    type Init = Addon;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.title.escape(),
            set_subtitle: &self.description.no_line_breaks().escape(),
            set_subtitle_lines: 1,
            set_selectable: false,
            set_activatable: true,
            connect_activated[sender, index] => move |_| {
                sender.output_sender().emit(AddonRowOutput::Clicked(index.current_index()));
            },

            add_prefix = &gtk::Box {
                add_css_class: "medium-icon",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_expand: false,
                set_height_request: 50,
                set_width_request: 50,
                set_margin_end: 6,
                set_margin_vertical: 12,
                set_overflow: gtk::Overflow::Hidden,

                #[local_ref]
                icon -> adw::Clamp,
            },

            add_suffix = &gtk::Image {
                set_margin_start: 6,
                set_icon_name: Some("right"),
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let icon = Image::builder()
            .launch(
                ImageInit::builder()
                    .source(init.icon)
                    .size(50)
                    .placeholder("puzzle-piece")
                    .build(),
            )
            .detach();

        Self {
            icon,
            title: init.name,
            description: init.description,
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let icon = self.icon.widget();
        let widgets = view_output!();

        widgets
    }
}
