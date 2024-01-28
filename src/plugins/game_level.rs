use bevy::prelude::*;

use crate::AppState;

pub struct GameLevelUiPlugin;

const BOTTOM_PANEL_HEIGHT: f32 = 200.;

#[derive(Debug, Component)]
struct GameUi;

fn add_game_level_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            GameUi,
        ))
        .with_children(|parent| {
            // spacer
            parent.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.,
                    ..Default::default()
                },
                ..Default::default()
            });
            // "time" label
            parent.spawn(
                TextBundle::from_section(
                    "Time",
                    TextStyle {
                        font_size: 32.,
                        ..Default::default()
                    },
                )
                .with_text_alignment(TextAlignment::Right)
                .with_background_color(Color::MIDNIGHT_BLUE)
                .with_style(Style {
                    width: Val::Percent(30.),
                    margin: UiRect {
                        left: Val::Percent(35.),
                        right: Val::Percent(35.),
                        ..Default::default()
                    },
                    align_self: AlignSelf::Center,
                    ..Default::default()
                }),
            );
            // bottom panel
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(BOTTOM_PANEL_HEIGHT),
                    width: Val::Percent(100.),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::WHITE),
                z_index: ZIndex::Global(-10),
                ..Default::default()
            });
        });
}

fn remove_game_level_ui(mut commands: Commands, query: Query<Entity, With<GameUi>>) {
    for ent in &query {
        commands.entity(ent).despawn();
    }
}

impl Plugin for GameLevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), add_game_level_ui)
            .add_systems(OnExit(AppState::InGame), remove_game_level_ui);
    }
}
