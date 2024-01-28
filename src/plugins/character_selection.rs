use crate::AppState::CharacterSelection;
use crate::{AppState, StartMenuTag};
use bevy::prelude::*;

#[derive(Debug)]
pub struct CharacterSelectionPlugin;

#[derive(Debug, Component)]
struct CharacterSelectionMenuTag;

impl Plugin for CharacterSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::CharacterSelection),
            (setup_character_menu, hover_feedback_system),
        )
        .add_systems(OnExit(CharacterSelection), cleanup_menu);
    }
}

const NORMAL_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const HOVERED_COLOR: Color = Color::rgb(1., 1., 1.);

fn setup_character_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let img_path = "images/ui/scenes/選角畫面.png";
    let bg_img = asset_server.load(img_path);

    commands.spawn(
        (SpriteBundle {
            texture: bg_img,
            transform: Transform {
                translation: Vec3::new(0., 0., -5.),
                ..default()
            },
            ..default()
        }, CharacterSelectionMenuTag),
    );

    let img_path = "images/ui/scenes/選角畫面_冥進_token.png";
    let left_party_img = asset_server.load(img_path);
    let left_pos = Vec3::new(-500., -75., 0.);
    commands.spawn(
        (SpriteBundle {
            sprite: Sprite {
                color: NORMAL_COLOR,
                ..default()
            },
            transform: Transform {
                translation: left_pos,
                scale: Vec3::new(1.05, 1.05, 1.05),
                ..default()
            },
            texture: left_party_img,
            ..default()
        }, CharacterSelectionMenuTag),
    );

    let img_path = "images/ui/scenes/選角畫面_大甲_token.png";
    let left_right_img = asset_server.load(img_path);
    let right_pos = Vec3::new(500., -75., 0.);
    commands.spawn(
        (SpriteBundle {
            sprite: Sprite {
                color: NORMAL_COLOR,
                ..default()
            },
            transform: Transform {
                translation: right_pos,
                scale: Vec3::new(1.05, 1.05, 1.05),
                ..default()
            },
            texture: left_right_img,
            ..default()
        }, CharacterSelectionMenuTag),
    );
}

fn hover_feedback_system(mut query: Query<(&Interaction, &mut Sprite), Changed<Interaction>>) {
    for (int, mut sprite) in &mut query {
        match *int {
            Interaction::Hovered => {
                sprite.color = HOVERED_COLOR;
            }
            Interaction::None => {
                sprite.color = NORMAL_COLOR;
            }
            _ => {}
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<CharacterSelectionMenuTag>>) {
    for ent in &query {
        commands.entity(ent).despawn_recursive();
    }
}
