mod config;
mod plugins;
mod ringcon;
mod sound_player;

use std::time::Duration;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::ringcon::RingConEvent;
use bevy::app::AppExit;
use bevy::input::{keyboard::KeyboardInput, ButtonState};
use bevy::math::bool;
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use config::ImageKey;
use plugins::art::ArtPlugin;
use plugins::character_selection::CharacterSelectionPlugin;
use plugins::game_level::GameLevelUiPlugin;
use plugins::start_menu::StartMenuPlugin;
use plugins::{JumpImage, JumpImagePlugin};
use sound_player::*;

#[derive(Debug, Event)]
struct GenEvent(i32, i32); //player/img

#[derive(Resource)]
pub struct ScoreSetting {
    pub basic_score: usize,
    pub combo_score: usize,
}

fn main() {
    App::new()
        .add_event::<AttackEvent>()
        .add_event::<GenEvent>()
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
        .add_plugins((
            JumpImagePlugin,
            GameLevelUiPlugin,
            SoundSystemPlugin,
            CharacterSelectionPlugin,
            StartMenuPlugin,
            #[cfg(target_os = "windos")]
            RingConPlugin,
        ))
        .add_plugins(ArtPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2dBundle::default());
        })
        .add_systems(OnEnter(AppState::InGame), setup_camera)
        .add_systems(
            FixedUpdate,
            (phah, score_system)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .insert_resource(CounterNumber {
            score1: 0,
            score2: 0,
        })
        .insert_resource(ComboNumber {
            score1: 0,
            score2: 0,
        })
        .insert_resource(ScoreSetting {
            basic_score: 3,
            combo_score: 1,
        })
        .add_systems(
            Update,
            (
                counter1_update_system,
                counter2_update_system,
                combo1_update_system,
                combo2_update_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    CharacterSelection,
    InGame,
}

fn phah(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    mut ringcon_evt: EventReader<RingConEvent>,
    mut query: Query<&mut SoundPlayer>,
    a: Res<ASound>,
    w: Res<WSound>,
    d: Res<DSound>,
    mut evt_w: EventWriter<AttackEvent>,
    mut sound_start_evt_w: EventWriter<SoundPlayerStart>,
    mut evt_exit: EventWriter<AppExit>,
) {
    for event in ringcon_evt.read() {
        for mut sound_player in &mut query {
            match sound_player.action.action_type {
                ActionType::Player1 => match event {
                    RingConEvent::Push => {
                        commands.spawn(AudioBundle {
                            source: a.0.clone(),
                            settings: PlaybackSettings::DESPAWN,
                        });
                        sound_player.key_down(1, &mut evt_w, true);
                    }
                    RingConEvent::Pull => {
                        commands.spawn(AudioBundle {
                            source: w.0.clone(),
                            settings: PlaybackSettings::DESPAWN,
                        });
                        sound_player.key_down(2, &mut evt_w, true);
                    }
                    RingConEvent::Squat => {
                        commands.spawn(AudioBundle {
                            source: d.0.clone(),
                            settings: PlaybackSettings::DESPAWN,
                        });
                        sound_player.key_down(3, &mut evt_w, true);
                    }
                },
                _ => {}
            }
        }
    }
    for event in events.read() {
        if event.state == ButtonState::Pressed {
            if event.key_code == Some(KeyCode::Space) {
                commands.spawn(JumpImage {
                    key: ImageKey::WhyHaRuHiKaGe,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }

            if event.key_code == Some(KeyCode::Escape) {
                evt_exit.send(AppExit);
            }

            if event.key_code == Some(KeyCode::G) {
                commands.spawn(JumpImage {
                    key: ImageKey::GenShinStart,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }
            if event.key_code == Some(KeyCode::H) {
                commands.spawn(JumpImage {
                    key: ImageKey::Monkey,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }
            if event.key_code == Some(KeyCode::J) {
                commands.spawn(JumpImage {
                    key: ImageKey::NetArmy,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }
            if event.key_code == Some(KeyCode::K) {
                commands.spawn(JumpImage {
                    key: ImageKey::MaZu,
                    from: Vec2::new(-960., 0.),
                    to: Vec2::new(-240., 0.),
                });
            }
            for mut sound_player in &mut query {
                match sound_player.action.action_type {
                    ActionType::Player1 => match event.key_code {
                        Some(KeyCode::A) => {
                            commands.spawn(AudioBundle {
                                source: a.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(1, &mut evt_w, false);
                        }
                        Some(KeyCode::W) => {
                            commands.spawn(AudioBundle {
                                source: w.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(2, &mut evt_w, false);
                        }
                        Some(KeyCode::D) => {
                            commands.spawn(AudioBundle {
                                source: d.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(3, &mut evt_w, false);
                        }
                        _ => {}
                    },
                    ActionType::Player2 => match event.key_code {
                        Some(KeyCode::G) => {
                            commands.spawn(AudioBundle {
                                source: a.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(1, &mut evt_w, false);
                        }
                        Some(KeyCode::Y) => {
                            commands.spawn(AudioBundle {
                                source: w.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(2, &mut evt_w, false);
                        }
                        Some(KeyCode::J) => {
                            commands.spawn(AudioBundle {
                                source: d.0.clone(),
                                settings: PlaybackSettings::DESPAWN,
                            });
                            sound_player.key_down(3, &mut evt_w, false);
                        }
                        _ => {}
                    },
                }

                if event.key_code == Some(KeyCode::O) {
                    println!("start");
                    sound_player.start(&mut sound_start_evt_w);
                }
            }
        }
    }
}

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

fn setup_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut evt_w: EventWriter<GenEvent>,
) {
    let background = asset_server.load("images/background.png");
    commands.spawn(SpriteBundle {
        texture: background,
        transform: Transform {
            translation: Vec3::new(0., 0., -10.),
            ..default()
        },
        ..Default::default()
    });
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
    evt_w.send(GenEvent(1, 3));
    evt_w.send(GenEvent(2, 3));
}

pub fn score_system(
    mut counter: ResMut<CounterNumber>,
    mut combo: ResMut<ComboNumber>,
    setting: Res<ScoreSetting>,
    mut evt_r: EventReader<AttackEvent>,
    mut evt_w: EventWriter<GenEvent>,
) {
    let mut increase_num;
    let mut gen_num;
    let mut gen_num2;
    for e in evt_r.read() {
        if e.0 == 1 {
            if e.1 {
                increase_num = combo.score1 + setting.basic_score;

                gen_num = ((counter.score1 + increase_num) / 5) - (counter.score1 / 5);
                gen_num2 = ((counter.score1 + increase_num) / 10) - (counter.score1 / 10);
                if gen_num >= 1 {
                    for _ in 0..gen_num {
                        evt_w.send(GenEvent(1, 1));
                        println!("evt_w.send gen={}", gen_num);
                    }
                }
                if gen_num >= 2 {
                    for _ in 0..gen_num {
                        evt_w.send(GenEvent(1, 2));
                        println!("evt_w.send gen={}", gen_num);
                    }
                }
                counter.score1 += combo.score1 + setting.basic_score;
                combo.score1 += setting.combo_score;
            } else {
                combo.score1 = 0;
                gen_num = 0;

                // TODO: check if miss
            }
        } else if e.1 {
            increase_num = combo.score2 + setting.basic_score;
            gen_num = ((counter.score2 + increase_num) / 5) - (counter.score2 / 5);
            gen_num2 = ((counter.score2 + increase_num) / 10) - (counter.score2 / 10);
            if gen_num >= 1 {
                for _ in 0..gen_num {
                    evt_w.send(GenEvent(2, 1));
                    println!("evt_w.send gen={}", gen_num);
                }
            }
            if gen_num2 >= 2 {
                for _ in 0..gen_num {
                    evt_w.send(GenEvent(2, 2));
                    println!("evt_w.send gen={}", gen_num);
                }
            }
        } else {
            combo.score2 = 0;
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
