use adw::prelude::*;
use chrono::{Datelike, Utc};
use itertools::Itertools;
use relm4::{adw, gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::constants::{APP_ID, APP_NAME};

pub struct AboutDialog {}

impl SimpleComponent for AboutDialog {
    type Init = ();
    type Widgets = adw::AboutDialog;
    type Input = ();
    type Output = ();
    type Root = adw::AboutDialog;

    fn init_root() -> Self::Root {
        let authors = env!("CARGO_PKG_AUTHORS").split(':').collect_vec();
        let copyright = format!("© {} {}", Utc::now().year(), authors[0]);

        adw::AboutDialog::builder()
            .application_icon(APP_ID)
            .application_name(APP_NAME)
            .version(env!("CARGO_PKG_VERSION"))
            .website(env!("CARGO_PKG_REPOSITORY"))
            .issue_url(env!("CARGO_PKG_REPOSITORY"))
            .license_type(gtk::License::Gpl30Only)
            .copyright(copyright)
            .developers(&*authors)
            .designers(&*authors)
            .build()
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = root.clone();

        ComponentParts { model, widgets }
    }

    fn update_view(&self, dialog: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let window = &relm4::main_application().windows()[0];
        dialog.present(Some(window));
    }
}
