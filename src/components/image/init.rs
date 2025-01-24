use relm4::gtk;
use url::Url;

pub trait ImageSize {
    fn into(self) -> (i32, i32);
}

impl ImageSize for i32 {
    fn into(self) -> (i32, i32) {
        (self, self)
    }
}

impl ImageSize for (i32, i32) {
    fn into(self) -> (i32, i32) {
        self
    }
}

pub struct ImageInit {
    pub source: Option<Url>,
    pub preload: bool,
    pub height: i32,
    pub width: i32,
    pub align: gtk::Align,
    pub content_fit: gtk::ContentFit,
    pub placeholder: Option<&'static str>,
    pub placeholder_size: Option<i32>,
}

impl ImageInit {
    pub fn builder() -> Self {
        Self {
            source: None,
            preload: true,
            height: 50,
            width: 50,
            align: gtk::Align::Center,
            content_fit: gtk::ContentFit::Contain,
            placeholder: None,
            placeholder_size: Some(30),
        }
    }

    pub fn source(mut self, value: Option<Url>) -> Self {
        self.source = value;
        self
    }

    pub fn preload(mut self, value: bool) -> Self {
        self.preload = value;
        self
    }

    pub fn size(mut self, value: impl ImageSize) -> Self {
        let (height, width) = value.into();
        self.height = height;
        self.width = width;
        self
    }

    pub fn align(mut self, value: gtk::Align) -> Self {
        self.align = value;
        self
    }

    pub fn content_fit(mut self, value: gtk::ContentFit) -> Self {
        self.content_fit = value;
        self
    }

    pub fn placeholder(mut self, value: &'static str) -> Self {
        self.placeholder = Some(value);
        self
    }

    pub fn placeholder_size(mut self, value: i32) -> Self {
        self.placeholder_size = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
