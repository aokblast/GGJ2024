use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::AppState;

pub struct GameLevelUiPlugin;

const BOTTOM_PANEL_HEIGHT: f32 = 200.;

#[derive(Debug, Component)]
struct GameUi;

fn add_game_level_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = asset_server.load("images/background.png");
    commands.spawn((
        SpriteBundle {
            texture: background,
            transform: Transform {
                translation: Vec3::new(0., 0., -10.),
                ..default()
            },
            ..Default::default()
        },
        GameUi,
    ));
    // time label
    // bottom panel
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                custom_size: Some(Vec2::new(1920., BOTTOM_PANEL_HEIGHT)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., -540., 0.),
                ..default()
            },
            // z_index: ZIndex::Global(2),
            ..Default::default()
        },
        GameUi,
    ));
}

fn remove_game_level_ui<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for ent in &query {
        commands.entity(ent).despawn();
    }
}

impl Plugin for GameLevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), add_game_level_ui)
            .add_systems(OnExit(AppState::InGame), remove_game_level_ui::<GameUi>);
    }
}
