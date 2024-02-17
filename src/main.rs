mod config;
mod plugins;

use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use plugins::art::ArtPlugin;
use plugins::character_selection::CharacterSelectionPlugin;
use plugins::game_level::GameLevelUiPlugin;
use plugins::input::GameInputPlugin;
use plugins::ringcon::RingConPlugin;
use plugins::score::ScorePlugin;
use plugins::sound_player::SoundSystemPlugin;
use plugins::start_menu::StartMenuPlugin;
use plugins::JumpImagePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // resolution: (1920., 1080.).into(),
                        // resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    level: log::Level::DEBUG,
                    ..Default::default()
                }),
        )
        .add_state::<AppState>()
        // third-party plugins
        .add_plugins(TweeningPlugin)
        // our plugins
        .add_plugins((
            JumpImagePlugin,
            GameLevelUiPlugin,
            SoundSystemPlugin,
            CharacterSelectionPlugin,
            StartMenuPlugin,
            GameInputPlugin,
            ArtPlugin,
            ScorePlugin,
            #[cfg(all(target_os = "windows", feature = "ringcon"))]
            RingConPlugin,
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2dBundle::default());
        })
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    CharacterSelection,
    InGame,
}
