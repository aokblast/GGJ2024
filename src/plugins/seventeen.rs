use bevy::{app::PluginGroupBuilder, prelude::*};

use super::{
    art::ArtPlugin, character_selection::CharacterSelectionPlugin, game_level::GameLevelUiPlugin,
    input::GameInputPlugin, score::ScorePlugin, sound_player::SoundSystemPlugin,
    start_menu::StartMenuPlugin, JumpImagePlugin,
};

#[derive(Debug)]
pub struct SeventeenPlugins;

impl PluginGroup for SeventeenPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>();

        // default plugins
        builder = builder
            .add(JumpImagePlugin)
            .add(GameLevelUiPlugin)
            .add(SoundSystemPlugin)
            .add(CharacterSelectionPlugin)
            .add(StartMenuPlugin)
            .add(GameInputPlugin)
            .add(ArtPlugin)
            .add(ScorePlugin);

        #[cfg(all(target_os = "windows", feature = "ringcon"))]
        {
            use super::ringcon::RingConPlugin;
            builder = builder.add(RingConPlugin)
        }

        builder
    }
}
