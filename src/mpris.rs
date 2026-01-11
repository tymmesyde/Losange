use mpris_server::{Metadata, PlaybackStatus, Player};
use relm4::gtk::glib;
use tracing::error;
use url::Url;

pub struct MPris {
    player: Player,
}

impl MPris {
    pub async fn new(id: &str, name: &str) -> Self {
        let player = Player::builder(name)
            .identity(name)
            .desktop_entry(id)
            .can_play(true)
            .can_pause(true)
            .can_raise(true)
            .can_go_previous(false)
            .can_go_next(false)
            .build()
            .await
            .expect("Failed to start MPRIS server");

        glib::MainContext::default().spawn_local(player.run());

        player
            .set_playback_status(PlaybackStatus::Playing)
            .await
            .ok();

        Self { player }
    }

    pub async fn set_playback_status(&self, paused: bool) {
        let status = match paused {
            true => PlaybackStatus::Paused,
            false => PlaybackStatus::Playing,
        };

        if let Err(e) = self.player.set_playback_status(status).await {
            error!("Failed to set mpris playback status: {e}");
        }
    }

    pub async fn set_metadata(&self, title: String, image: Option<Url>) {
        let mut metadata = Metadata::builder().title(title).artist(vec![""]).build();

        let art_url = image.map(|image| image.to_string());
        metadata.set_art_url(art_url);

        if let Err(e) = self.player.set_metadata(metadata).await {
            error!("Failed to set mpris metadata: {e}");
        }
    }

    pub fn on_play_pause<F: Fn() + 'static>(&self, callback: F) {
        self.player.connect_play_pause(move |_| {
            callback();
        });
    }

    pub fn on_raise<F: Fn() + 'static>(&self, callback: F) {
        self.player.connect_raise(move |_| {
            callback();
        });
    }
}
