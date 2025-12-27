use adw::prelude::*;
use relm4::{adw, ComponentParts, ComponentSender, SimpleComponent};
use rust_i18n::t;

pub struct ShortcutsDialog {}

impl SimpleComponent for ShortcutsDialog {
    type Root = adw::ShortcutsDialog;
    type Widgets = adw::ShortcutsDialog;
    type Init = ();
    type Input = ();
    type Output = ();

    fn init_root() -> Self::Root {
        adw::ShortcutsDialog::builder().build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = root.clone();

        let section = adw::ShortcutsSection::new(None);
        section.add(adw::ShortcutsItem::new(
            &t!("shortcut_search"),
            "<Control>F",
        ));
        section.add(adw::ShortcutsItem::new(
            &t!("shortcut_preferences"),
            "<Control>comma",
        ));

        widgets.add(section);

        ComponentParts { model, widgets }
    }

    fn update_view(&self, dialog: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let window = &relm4::main_application().windows()[0];
        dialog.present(Some(window));
    }
}
