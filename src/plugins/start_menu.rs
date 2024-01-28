use crate::AppState;
use bevy::prelude::*;

#[derive(Debug)]
pub struct StartMenuPlugin;

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

#[derive(Debug, Component)]
struct StartMenuTag;

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // background image
    let img_path = "images/ui/scenes/起始畫面.png";
    let img = asset_server.load(img_path);
    commands.spawn((
        SpriteBundle {
            texture: img,
            transform: Transform {
                translation: Vec3::new(0., 0., -10.),
                ..default()
            },
            ..default()
        },
        StartMenuTag,
    ));

    // "play" button
    let img_path = "images/ui/scenes/起始畫面_play_token.png";
    let img = asset_server.load(img_path);
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        // HACK: hard-coded size
                        style: Style {
                            width: Val::Px(206.),
                            height: Val::Px(100.),
                            margin: UiRect {
                                top: Val::Px(240.),
                                ..default()
                            },
                            ..default()
                        },
                        image: UiImage {
                            texture: img,
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(0., -187., 0.),
                            ..default()
                        },
                        ..default()
                    },
                    StartMenuTag,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        " ",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, _color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // next_state.set(AppState::CharacterSelection);
                next_state.set(AppState::InGame);
            }
            _ => {}
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<StartMenuTag>>) {
    for ent in &query {
        commands.entity(ent).despawn_recursive();
    }
}
