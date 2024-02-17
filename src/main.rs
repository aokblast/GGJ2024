mod config;
mod plugins;

use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use plugins::seventeen::SeventeenPlugins;

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
        .add_plugins(SeventeenPlugins)
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
