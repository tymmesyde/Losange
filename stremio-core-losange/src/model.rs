use stremio_core::models::addon_details::AddonDetails;
use stremio_core::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::installed_addons_with_filters::InstalledAddonsWithFilters;
use stremio_core::models::library_with_filters::{LibraryWithFilters, NotRemovedFilter};
use stremio_core::models::meta_details::MetaDetails;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::Effects;
use stremio_core::types::addon::Descriptor;
use stremio_core::types::events::DismissedEventsBucket;
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::notifications::NotificationsBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::MetaItemPreview;
use stremio_core::types::search_history::SearchHistoryBucket;
use stremio_core::types::server_urls::ServerUrlsBucket;
use stremio_core::types::streams::StreamsBucket;
use stremio_core::Model;

use crate::env::LosangeEnv;
use crate::models;

#[derive(Model, Clone)]
#[model(LosangeEnv)]
pub struct LosangeModel {
    pub ctx: Ctx,
    pub continue_watching: ContinueWatchingPreview,
    pub home: CatalogsWithExtra,
    pub discover: CatalogWithFilters<MetaItemPreview>,
    pub library: LibraryWithFilters<NotRemovedFilter>,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub installed_addons: InstalledAddonsWithFilters,
    pub remote_addons: CatalogWithFilters<Descriptor>,
    pub addon_details: AddonDetails,
    pub player: Player,
    pub server: StreamingServer,
}

impl LosangeModel {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        streams: StreamsBucket,
        streaming_server_urls: ServerUrlsBucket,
        notifications: NotificationsBucket,
        search_history: SearchHistoryBucket,
        dismissed_events: DismissedEventsBucket,
    ) -> (LosangeModel, Effects) {
        let (continue_watching, continue_watching_effects) =
            ContinueWatchingPreview::new(&library, &notifications);
        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&profile);
        let (library_, library_effects) = LibraryWithFilters::new(&library, &notifications);
        let (installed_addons, installed_addons_effects) =
            InstalledAddonsWithFilters::new(&profile);
        let (remote_addons, remote_addons_effects) =
            CatalogWithFilters::<Descriptor>::new(&profile);
        let (server, server_effects) = StreamingServer::new::<LosangeEnv>(&profile);

        let ctx = Ctx::new(
            profile,
            library,
            streams,
            streaming_server_urls,
            notifications,
            search_history,
            dismissed_events,
        );

        let model = LosangeModel {
            ctx,
            continue_watching,
            home: Default::default(),
            discover,
            library: library_,
            search: Default::default(),
            meta_details: Default::default(),
            player: Default::default(),
            installed_addons,
            remote_addons,
            addon_details: Default::default(),
            server,
        };

        (
            model,
            continue_watching_effects
                .join(discover_effects)
                .join(library_effects)
                .join(remote_addons_effects)
                .join(installed_addons_effects)
                .join(server_effects),
        )
    }

    pub fn update(&self, fields: Vec<LosangeModelField>) {
        for field in fields {
            match field {
                LosangeModelField::Ctx => models::ctx::update(&self.ctx),
                LosangeModelField::ContinueWatching | LosangeModelField::Home => {
                    models::home::update(&self.home, &self.continue_watching, &self.ctx)
                }
                LosangeModelField::Discover => models::discover::update(&self.discover),
                LosangeModelField::Library => models::library::update(&self.library),
                LosangeModelField::Search => models::search::update(&self.search, &self.ctx),
                LosangeModelField::MetaDetails => {
                    models::meta_details::update(&self.meta_details, &self.ctx)
                }
                LosangeModelField::InstalledAddons => {
                    models::installed_addons::update(&self.installed_addons)
                }
                LosangeModelField::RemoteAddons => {
                    models::remote_addons::update(&self.remote_addons)
                }
                LosangeModelField::AddonDetails => {
                    models::addon_details::update(&self.installed_addons, &self.addon_details)
                }
                LosangeModelField::Player => models::player::update(&self.player, &self.ctx),
                LosangeModelField::Server => models::server::update(&self.server),
            }
        }
    }
}
