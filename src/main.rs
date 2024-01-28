mod config;
mod plugins;
mod ringcon;
mod sound_player;

use std::thread;
use std::time::Duration;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::query;
use bevy::math::bool;
use bevy::prelude::*;
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_tweening::TweeningPlugin;
use config::ImageKey;
use dlopen2::wrapper::Container;
use plugins::game_level::GameLevelUiPlugin;
use plugins::{JumpImage, JumpImagePlugin};
use ringcon::{test_ringcon, PullVal, RingConApi};
use sound_player::*;

#[derive(Resource)]
pub struct WSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct ASound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct DSound(pub Handle<AudioSource>);

fn main() {
    App::new()
        .add_event::<AttackEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1920., 1080.).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_state::<AppState>()
        // third-party plugins
        .add_plugins(TweeningPlugin)
        // our plugins
        .add_plugins((JumpImagePlugin, GameLevelUiPlugin, SoundSystemPlugin))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), setup_camera)
        .add_systems(
            FixedUpdate,
            (phah, score_system)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, sound_timer)
        //.run_if(in_state(AppState::InGame))
        .insert_resource(CounterNumber {
            score1: 0,
            score2: 0,
        })
        .insert_resource(ComboNumber {
            score1: 0,
            score2: 0,
        })
        //.add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            ((
                counter1_update_system,
                counter2_update_system,
                combo1_update_system,
                combo2_update_system,
            ))
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sound1 = Sound(asset_server.load("sounds/gong.ogg"));
    let sound2 = Sound(asset_server.load("sounds/gong.ogg"));
    let t11 = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 100.0,
            color: Color::ORANGE,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(150.0),
        left: Val::Px(5.0),
        ..default()
    });
    let t12 = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 100.0,
            color: Color::ORANGE,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(250.0),
        left: Val::Px(5.0),
        ..default()
    });
    let t21 = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 100.0,
            color: Color::ORANGE,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(150.0),
        right: Val::Px(5.0),
        ..default()
    });
    let t22 = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 100.0,
            color: Color::ORANGE,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(250.0),
        right: Val::Px(5.0),
        ..default()
    });

    let sound1_id = commands.spawn(sound1).id();
    let sound2_id = commands.spawn(sound2).id();
    let t11_id = commands.spawn(t11).id();
    let t12_id = commands.spawn(t12).id();
    let t21_id = commands.spawn(t21).id();
    let t22_id = commands.spawn(t22).id();

    let sound_interval = 1000;
    let sound_player1: SoundPlayer = SoundPlayer::new(
        sound_interval,
        ActionType::Player1,
        sound1_id,
        t11_id,
        t12_id,
    );
    let sound_player2: SoundPlayer = SoundPlayer::new(
        sound_interval,
        ActionType::Player2,
        sound2_id,
        t21_id,
        t22_id,
    );

    // attach beat timer to sound player
    // TODO: move this system to SoundSystemPlugin
    let sound_timer = Timer::new(
        Duration::from_millis(sound_interval as u64),
        TimerMode::Repeating,
    );
    commands
        .spawn(sound_player1)
        .insert(BeatTimer(sound_timer.clone()));
    commands.spawn(sound_player2).insert(BeatTimer(sound_timer));

    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(ASound(asset_server.load("sounds/A.ogg")));
    commands.insert_resource(WSound(asset_server.load("sounds/W.ogg")));
    commands.insert_resource(DSound(asset_server.load("sounds/D.ogg")));
}

#[derive(Resource)]
struct MenuData {
    //game_title_entity:Entity,
    button_single_player_entity: Entity,
    button_double_player_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu(mut commands: Commands) {
    let game_title_ = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "GAME NAME HERE",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
    let button_single_player_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.),
                        height: Val::Px(100.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Single player",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    let button_double_player_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.),
                        height: Val::Px(100.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Double player",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(MenuData {
        button_single_player_entity,
        button_double_player_entity,
    });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands
        .entity(menu_data.button_single_player_entity)
        .despawn_recursive();
    commands
        .entity(menu_data.button_double_player_entity)
        .despawn_recursive();
}

fn phah(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    mut query: Query<&mut SoundPlayer>,
    a: Res<ASound>,
    w: Res<WSound>,
    d: Res<DSound>,
    mut evt_w: EventWriter<AttackEvent>,
    mut sound_start_evt_w: EventWriter<SoundPlayerStart>,
) {
    for event in events.read() {
        if event.state == ButtonState::Pressed {
            if event.key_code == Some(KeyCode::Space) {
                commands.spawn(JumpImage {
                    key: ImageKey::GenShinStart,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }
            for mut sound_player in &mut query {
                let is_player_1 = sound_player.action.action_type == ActionType::Player1;
                if event.key_code
                    == if is_player_1 {
                        Some(KeyCode::A)
                    } else {
                        Some(KeyCode::G)
                    }
                {
                    commands.spawn(AudioBundle {
                        source: a.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                    });
                    sound_player.key_down(1, &mut evt_w);
                }
                if event.key_code
                    == if is_player_1 {
                        Some(KeyCode::W)
                    } else {
                        Some(KeyCode::Y)
                    }
                {
                    commands.spawn(AudioBundle {
                        source: w.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                    });
                    sound_player.key_down(2, &mut evt_w);
                }
                if event.key_code
                    == if is_player_1 {
                        Some(KeyCode::D)
                    } else {
                        Some(KeyCode::J)
                    }
                {
                    commands.spawn(AudioBundle {
                        source: d.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                    });
                    sound_player.key_down(3, &mut evt_w);
                }
                if event.key_code == Some(KeyCode::O) {
                    println!("start");
                    sound_player.start(&mut sound_start_evt_w);
                }
            }
        }
    }
}

fn sound_timer(
    mut commands: Commands,
    mut query: Query<&mut SoundPlayer>,
    mut text_query: Query<&mut Text>,
    sound_query: Query<&Sound>,
) {
    for mut sound_player in &mut query {
        if sound_player.update() {
            if let Ok(sound) = sound_query.get_component::<Sound>(sound_player.sound_id) {
                commands.spawn(AudioBundle {
                    source: sound.0.clone(),
                    // auto-despawn the entity when playback finishes
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
        let mut s = "".to_owned();
        for k in &sound_player.past_key {
            s += k.to_string().as_str();
        }
        for _ in sound_player.past_key.len()..sound_player.action.keys.len() {
            s += " ";
        }
        if let Ok(mut text) = text_query.get_component_mut::<Text>(sound_player.past_text_id) {
            text.sections[0].value = s;
        }

        s = "".to_owned();
        for k in &sound_player.action.keys {
            s += k.to_string().as_str();
        }
        if let Ok(mut text) = text_query.get_component_mut::<Text>(sound_player.goal_text_id) {
            text.sections[0].value = s;
        }
    }
}

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
struct Colortext;

const COUNTER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const SCOREBOARD_FONT_SIZE: f32 = 40.0;

#[derive(Component)]
struct CounterText1;

#[derive(Component)]
struct CounterText2;

#[derive(Debug, Event)]
struct AttackEvent(i32, bool);

#[derive(Component)]
struct PastKeys;

#[derive(Component)]
struct ComboText1;

#[derive(Component)]
struct ComboText2;

#[derive(Resource)]
pub struct CounterNumber {
    pub score1: usize,
    pub score2: usize,
}

#[derive(Resource)]
pub struct ComboNumber {
    pub score1: usize,
    pub score2: usize,
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = asset_server.load("images/background.png");
    commands.spawn(SpriteBundle {
        texture: background,
        transform: Transform {
            translation: Vec3::new(0., 0., -10.),
            ..default()
        },
        ..Default::default()
    });
    // commands.spawn((
    //     TextBundle::from_section(
    //         "PaTaPon!",
    //         TextStyle {
    //             font_size: 100.0,
    //             ..default()
    //         },
    //     )
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         bottom: Val::Px(5.0),
    //         top: Val::Px(5.0),
    //         ..default()
    //     }),
    //     Colortext,
    // ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Player1\nScore: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: COUNTER_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        CounterText1,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Combo: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: COUNTER_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            left: Val::Px(5.0),
            ..default()
        }),
        ComboText1,
    ));
    commands.spawn((TextBundle::from_sections([
        TextSection::new(
            "Player2:",
            TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            },
        ),
        TextSection::from_style(TextStyle {
            font_size: SCOREBOARD_FONT_SIZE,
            color: COUNTER_COLOR,
            ..default()
        }),
    ])
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(5.0),
        right: Val::Px(5.0),
        ..default()
    }),));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: COUNTER_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(5.0),
            ..default()
        }),
        CounterText2,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Combo: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: COUNTER_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            right: Val::Px(5.0),
            ..default()
        }),
        ComboText2,
    ));
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<Colortext>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

pub fn score_system(
    mut counter: ResMut<CounterNumber>,
    mut combo: ResMut<ComboNumber>,
    mut evt: EventReader<AttackEvent>,
) {
    for e in evt.read() {
        if e.0 == 1 {
            if e.1 {
                counter.score1 += combo.score1 + 3;
                combo.score1 += 1;
            } else {
                combo.score1 = 0;
            }
        } else {
            if e.1 {
                counter.score2 += combo.score2 + 3;
                combo.score2 += 1;
            } else {
                combo.score2 = 0;
            }
        }
    }
}

fn counter1_update_system(
    counter: Res<CounterNumber>,
    mut query: Query<&mut Text, With<CounterText1>>,
) {
    for mut text in &mut query {
        text.sections[1].value = counter.score1.to_string();
    }
}
fn counter2_update_system(
    counter: Res<CounterNumber>,
    mut query: Query<&mut Text, With<CounterText2>>,
) {
    for mut text in &mut query {
        text.sections[1].value = counter.score2.to_string();
    }
}

fn combo1_update_system(oombo: Res<ComboNumber>, mut query: Query<&mut Text, With<ComboText1>>) {
    for mut text in &mut query {
        text.sections[1].value = oombo.score1.to_string();
    }
}
fn combo2_update_system(combo: Res<ComboNumber>, mut query: Query<&mut Text, With<ComboText2>>) {
    for mut text in &mut query {
        text.sections[1].value = combo.score2.to_string();
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)] //+, Reflect, Serialize, Deserialize#[reflect(Serialize, Deserialize)]
pub enum JustifyText {
    /// Leftmost character is immediately to the right of the render position.
    /// Bounds start from the render position and advance rightwards.
    #[default]
    Left,
    /// Leftmost & rightmost characters are equidistant to the render position.
    /// Bounds start from the render position and advance equally left & right.
    Center,
    /// Rightmost character is immediately to the left of the render position.
    /// Bounds start from the render position and advance leftwards.
    Right,
}
