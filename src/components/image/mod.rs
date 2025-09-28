pub mod init;

use gtk::gdk;
use gtk::prelude::*;
use init::ImageInit;
use relm4::{
    adw,
    component::{AsyncComponent, AsyncComponentParts},
    gtk, AsyncComponentSender, RelmWidgetExt,
};
use url::Url;

use crate::common::{image, net::fetch};

#[derive(Debug)]
pub enum ImageInput {
    Load,
    Unload,
    Update(Option<Url>),
}

#[derive(Debug)]
pub struct Image {
    source: Option<Url>,
    size: (i32, i32),
    align: gtk::Align,
    content_fit: gtk::ContentFit,
    placeholder: Option<&'static str>,
    placeholder_size: Option<i32>,
    paintable: Option<gdk::Texture>,
}

#[relm4::component(async pub)]
impl AsyncComponent for Image {
    type Input = ImageInput;
    type Output = ();
    type Init = ImageInit;
    type CommandOutput = anyhow::Result<gdk::Texture>;

    view! {
        adw::Clamp {
            set_align: model.align,
            set_maximum_size: model.size.0,

            gtk::Box {
                set_expand: true,
                set_align: model.align,
                set_height_request: model.size.1,

                #[transition = "Crossfade"]
                match &model.paintable {
                    Some(paintable) => gtk::Picture {
                        #[watch]
                        set_paintable: Some(paintable),
                        set_content_fit: model.content_fit,
                    },
                    None => gtk::Image {
                        set_visible: model.placeholder.is_some(),
                        set_icon_name: model.placeholder,
                        set_pixel_size: model.placeholder_size.unwrap_or(0),
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Image {
            source: init.source.to_owned(),
            size: (init.width, init.height),
            align: init.align,
            content_fit: init.content_fit,
            placeholder: init.placeholder,
            placeholder_size: init.placeholder_size,
            paintable: None,
        };

        let widgets = view_output!();

        if init.preload {
            sender.input_sender().emit(ImageInput::Load);
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ImageInput::Load => {
                if self.paintable.is_none() {
                    sender.oneshot_command(Self::load(self.source.to_owned(), self.size));
                }
            }
            ImageInput::Unload => {
                if self.paintable.is_some() {
                    self.paintable = None;
                }
            }
            ImageInput::Update(url) => {
                sender.oneshot_command(Self::load(url, self.size));
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.paintable = message.ok();
    }
}

impl Image {
    async fn load(source: Option<Url>, size: (i32, i32)) -> anyhow::Result<gdk::Texture> {
        if let Some(uri) = source {
            let response = fetch(uri).await?;
            let bytes = response.bytes().await?;

            let pixbuf = image::pixbuf_from_bytes(bytes)?;
            let scaled_pixbuf = image::scale_pixbuf(pixbuf, size);

            if let Some(pixbuf) = scaled_pixbuf {
                return Ok(gdk::Texture::for_pixbuf(&pixbuf));
            }
        }

        Err(anyhow::Error::msg("Failed to load image"))
    }
}
