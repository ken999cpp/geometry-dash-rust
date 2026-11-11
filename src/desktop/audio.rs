use super::{Game, Screen};
use crate::storage::MusicChannel;
use bevy::audio::Volume;
use bevy::prelude::*;
use std::time::Duration;

pub struct GameAudioPlugin;
#[derive(Component)]
pub(super) struct Music;
#[derive(Resource, Default)]
struct AudioState {
    reference: String,
}
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioState>()
            .add_systems(Update, sync_audio.in_set(super::FrameSet::Audio));
    }
}
pub(super) fn audio_path(reference: &str) -> String {
    let path = crate::level::asset_path(reference);
    if path.ends_with(".mp3") || path.ends_with(".wav") {
        format!("{}.ogg", path.rsplit_once('.').unwrap().0)
    } else {
        path
    }
}
fn music_paused(game: &Game) -> bool {
    game.exit_dialog.is_some()
        || matches!(
            game.screen,
            Screen::Loading | Screen::Paused | Screen::Complete
        )
        || (game.screen == Screen::Playing
            && (!game.attempt.playing()
                || game.block_fixed_frame
                || game
                    .simulation
                    .as_ref()
                    .is_some_and(|sim| matches!(sim.status, crate::simulation::Status::Dead(_)))))
}
fn sync_audio(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut state: ResMut<AudioState>,
    server: Res<AssetServer>,
    mut music: Query<(Entity, Option<&mut AudioSink>, &mut PlaybackSettings), With<Music>>,
) {
    let paused = music_paused(&game);
    let channel = if matches!(
        game.screen,
        Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
    ) {
        MusicChannel::Level
    } else {
        MusicChannel::Menu
    };
    let volume = Volume::Linear(game.profile.settings.volume(channel) as f32 / 100.);
    let reference = if matches!(
        game.screen,
        Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
    ) {
        game.simulation
            .as_ref()
            .map(|s| s.level.music.as_str())
            .unwrap_or("")
    } else {
        "assets/music/menuLoop.mp3"
    };
    let replace = state.reference != reference || game.pending_seek.is_some();
    if replace {
        let reference = reference.to_owned();
        for (e, _, _) in &music {
            commands.entity(e).despawn();
        }
        let offset = game.pending_seek.take().unwrap_or(0.);
        if !reference.is_empty() {
            let menu = !matches!(
                game.screen,
                Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
            );
            let mut settings = if menu {
                PlaybackSettings::LOOP
            } else {
                PlaybackSettings::ONCE
            };
            settings.paused = paused;
            settings.volume = volume;
            // Vorbis's lightweight decoder cannot seek an existing sink. Recreate
            // playback at the offset; Bevy skips samples before starting the sink.
            settings.start_position = Some(Duration::from_secs_f64(offset.max(0.)));
            commands.spawn((
                Music,
                AudioPlayer::new(server.load(audio_path(&reference))),
                settings,
            ));
        }
        state.reference = reference;
    } else {
        for (_, sink, mut settings) in &mut music {
            // Also cover music whose asynchronous decoder has not started yet.
            settings.paused = paused;
            settings.volume = volume;
            if let Some(mut sink) = sink {
                if sink.volume() != volume {
                    sink.set_volume(volume);
                }
                if paused {
                    sink.pause();
                } else {
                    sink.play();
                }
            }
        }
    }
    if let Some(sfx) = game.sfx.take() {
        commands.spawn((
            AudioPlayer::new(server.load(format!("sfx/{sfx}.ogg"))),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        level::{GROUND, Level, LevelObject},
        simulation::{Simulation, Status},
    };
    #[test]
    fn menu_and_level_volumes_update_pending_playback_without_restarting() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.profile = crate::storage::Profile::default();
        game.profile.settings.menu_music_volume = 37;
        game.profile.settings.level_music_volume = 73;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<AudioSource>()
            .insert_resource(game)
            .insert_resource(AudioState {
                reference: "assets/music/menuLoop.mp3".into(),
            })
            .add_systems(Update, sync_audio);
        let music = app.world_mut().spawn((Music, PlaybackSettings::LOOP)).id();
        app.update();
        assert_eq!(
            app.world().get::<PlaybackSettings>(music).unwrap().volume,
            Volume::Linear(0.37)
        );
        app.world_mut()
            .resource_mut::<Game>()
            .profile
            .settings
            .menu_music_volume = 0;
        app.update();
        assert_eq!(
            app.world().get::<PlaybackSettings>(music).unwrap().volume,
            Volume::Linear(0.)
        );
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            let mut level = Level::empty("Same track, separate volume");
            level.music = "assets/music/menuLoop.mp3".into();
            game.simulation = Some(Simulation::new(level).unwrap());
            game.screen(Screen::Paused);
        }
        app.update();
        let playback = app.world().get::<PlaybackSettings>(music).unwrap();
        assert_eq!(playback.volume, Volume::Linear(0.73));
        assert!(playback.paused);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Music>>()
                .iter(app.world())
                .count(),
            1
        );
    }
    #[test]
    fn collision_pauses_pending_music_before_respawn_without_pausing_sfx() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let root = std::env::temp_dir().join(format!("gd-death-audio-{}", std::process::id()));
        game.root = root.clone();
        game.testing = true;
        game.screen = Screen::Playing;
        let mut level = Level::empty("Audio regression");
        level.objects.push(LevelObject::new("spike", 40., GROUND));
        game.simulation = Some(Simulation::new(level).unwrap());
        let mut app = App::new();
        app.insert_resource(game)
            .add_systems(Update, super::super::step);
        let music = app.world_mut().spawn((Music, PlaybackSettings::ONCE)).id();
        let sfx = app.world_mut().spawn(PlaybackSettings::ONCE).id();
        app.update();
        assert!(matches!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .status,
            Status::Dead(_)
        ));
        assert!(app.world().get::<PlaybackSettings>(music).unwrap().paused);
        assert!(!app.world().get::<PlaybackSettings>(sfx).unwrap().paused);
        assert!(music_paused(app.world().resource::<Game>()));
        for _ in 0..120 {
            app.update();
        }
        assert!(matches!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .status,
            Status::Dead(_)
        ));
        // A stale dead simulation must not pause music after returning to menus.
        app.world_mut().resource_mut::<Game>().screen = Screen::Menu;
        assert!(!music_paused(app.world().resource::<Game>()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
