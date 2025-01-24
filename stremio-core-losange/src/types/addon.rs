use stremio_core::types::addon::{Descriptor, DescriptorPreview};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct Addon {
    pub manifest_url: Url,
    pub configure_url: String,
    pub domain: String,
    pub icon: Option<Url>,
    pub name: String,
    pub description: String,
    pub official: bool,
    pub protected: bool,
    pub configurable: bool,
    pub configuration_required: bool,
    pub installed: bool,
}

impl From<&DescriptorPreview> for Addon {
    fn from(descriptor: &DescriptorPreview) -> Self {
        Self {
            manifest_url: descriptor.transport_url.to_owned(),
            configure_url: descriptor
                .transport_url
                .as_str()
                .replace("manifest.json", "configure"),
            domain: descriptor
                .transport_url
                .domain()
                .map(str::to_string)
                .unwrap_or_default(),
            icon: descriptor.manifest.logo.to_owned(),
            name: descriptor.manifest.name.to_owned(),
            description: descriptor
                .manifest
                .description
                .to_owned()
                .unwrap_or_default(),
            official: false,
            protected: true,
            configurable: descriptor.manifest.behavior_hints.configurable,
            configuration_required: descriptor.manifest.behavior_hints.configuration_required,
            installed: false,
        }
    }
}

impl From<&Descriptor> for Addon {
    fn from(descriptor: &Descriptor) -> Self {
        Self {
            manifest_url: descriptor.transport_url.to_owned(),
            configure_url: descriptor
                .transport_url
                .as_str()
                .replace("manifest.json", "configure"),
            domain: descriptor
                .transport_url
                .domain()
                .map(str::to_string)
                .unwrap_or_default(),
            icon: descriptor.manifest.logo.to_owned(),
            name: descriptor.manifest.name.to_owned(),
            description: descriptor
                .manifest
                .description
                .to_owned()
                .unwrap_or_default(),
            official: descriptor.flags.official,
            protected: descriptor.flags.protected,
            configurable: descriptor.manifest.behavior_hints.configurable,
            configuration_required: descriptor.manifest.behavior_hints.configuration_required,
            installed: false,
        }
    }
}
