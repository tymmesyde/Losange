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

        let general_section = adw::ShortcutsSection::new(Some(&t!("general")));
        general_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_search"),
            "<Control>F",
        ));
        general_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_preferences"),
            "<Control>comma",
        ));
        general_section.add(adw::ShortcutsItem::new(&t!("shortcut_quit"), "<Control>Q"));

        let player_section = adw::ShortcutsSection::new(Some(&t!("player")));
        player_section.add(adw::ShortcutsItem::new(&t!("shortcut_play_pause"), "space"));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_seek_backward"),
            "Left",
        ));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_seek_forward"),
            "Right",
        ));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_increase_volume"),
            "Up",
        ));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_decrease_volume"),
            "Down",
        ));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_toggle_fullscreen"),
            "F",
        ));
        player_section.add(adw::ShortcutsItem::new(
            &t!("shortcut_exit_fullscreen"),
            "Escape",
        ));

        widgets.add(general_section);
        widgets.add(player_section);

        ComponentParts { model, widgets }
    }

    fn update_view(&self, dialog: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let window = &relm4::main_application().windows()[0];
        dialog.present(Some(window));
    }
}
