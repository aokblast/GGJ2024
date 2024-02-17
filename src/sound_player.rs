use crate::plugins::input::{PlayerCommand, PlayerCommandEvent};
use crate::{AppState, AttackEvent};
use bevy::audio::{PlaybackMode, Volume};
use bevy::{log, prelude::*};
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseMethod, Tween};
use rand::{thread_rng, Rng};
use std::time::Duration;
use std::vec;

const BEAT_START: Vec2 = Vec2::new(0., -450.);
const BEAT_END_P1: Vec2 = Vec2::new(-700., -450.);
const BEAT_END_P2: Vec2 = Vec2::new(700., -450.);
const BEAT_RING_OFFSET: f32 = 100.;

#[derive(Debug)]
pub struct SoundSystemPlugin;

impl Plugin for SoundSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundPlayerStart>()
            .add_systems(Startup, setup_sound_system)
            .add_systems(OnEnter(AppState::InGame), start_sound_player)
            .add_systems(
                Update,
                ((
                    (check_key_down, sound_timer).chain(),
                    (
                        (produce_beat_system, produce_beat_on_player_start),
                        move_beat_system,
                    )
                        .chain(),
                    player_hit_sound_system,
                )
                    .chain())
                .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ActionType {
    Player1,
    Player2,
}

pub struct Action {
    pub keys: Vec<i32>,
    pub action_type: ActionType,
}

#[derive(Component)]
pub struct Sound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct WSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct ASound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct DSound(pub Handle<AudioSource>);

#[derive(Event)]
pub struct SoundPlayerStart(pub ActionType);

#[derive(Component)]
pub struct SoundPlayer {
    pub action: Action,
    timer: Timer,
    interval: Duration,
    pub past_key: Vec<i32>,
    pub sound_id: Entity,
    pub goal_text_id: Entity,
    pub past_text_id: Entity,
    pressed: bool,
}

impl Action {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            keys: vec![],
            action_type,
        }
    }
}

impl SoundPlayer {
    pub fn new(
        interval: Duration,
        action_type: ActionType,
        sound_id: Entity,
        goal_text_id: Entity,
        past_text_id: Entity,
    ) -> Self {
        let mut player = Self {
            action: Action::new(action_type),
            interval,
            timer: Timer::new(interval, TimerMode::Repeating),
            past_key: vec![],
            sound_id,
            goal_text_id,
            past_text_id,
            pressed: false,
        };
        player.reroll();
        player
    }

    fn reroll(&mut self) {
        self.action.keys.clear();
        let mut rng = thread_rng();
        let len = rng.gen_range(1..6);
        for _ in 0..len {
            self.action.keys.push(rng.gen_range(1..3));
        }
    }

    fn do_action(action_type: &ActionType, evt_w: &mut EventWriter<AttackEvent>) {
        match action_type {
            ActionType::Player1 => {
                log::info!("Player1 attack");
                evt_w.send(AttackEvent(1, true));
            }
            ActionType::Player2 => {
                log::info!("Player2 attack");
                evt_w.send(AttackEvent(2, true));
            }
        }
    }

    fn fail(&mut self, evt_w: &mut EventWriter<AttackEvent>) {
        evt_w.send(AttackEvent(
            if self.action.action_type == ActionType::Player1 {
                1
            } else {
                2
            },
            false,
        ));
        self.past_key.clear();
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta);
        if !self.timer.just_finished() {
            return false;
        }

        self.pressed = false;
        true
    }
}

fn setup_sound_system(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    let sound_interval = Duration::from_millis(1000);
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
    let sound_timer = Timer::new(sound_interval, TimerMode::Repeating);
    commands
        .spawn(sound_player1)
        .insert(BeatTimer(sound_timer.clone()));
    commands.spawn(sound_player2).insert(BeatTimer(sound_timer));

    commands.insert_resource(ASound(asset_server.load("sounds/A.ogg")));
    commands.insert_resource(WSound(asset_server.load("sounds/W.ogg")));
    commands.insert_resource(DSound(asset_server.load("sounds/D.ogg")));

    let ring_img = asset_server.load("images/ui/game/white.png");
    commands.spawn(SpriteBundle {
        texture: ring_img.clone(),
        transform: Transform {
            translation: BEAT_END_P1.extend(10.) + Vec3::new(BEAT_RING_OFFSET, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: ring_img,
        transform: Transform {
            translation: BEAT_END_P2.extend(10.) - Vec3::new(BEAT_RING_OFFSET, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn sound_timer(
    mut commands: Commands,
    mut query: Query<&mut SoundPlayer>,
    mut text_query: Query<&mut Text>,
    sound_query: Query<&Sound>,
    time: Res<Time>,
) {
    for mut sound_player in &mut query {
        if sound_player.update(time.delta()) {
            if let Ok(sound) = sound_query.get_component::<Sound>(sound_player.sound_id) {
                commands.spawn(AudioBundle {
                    source: sound.0.clone(),
                    // auto-despawn the entity when playback finishes
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            log::debug!("timer just finished");
        }

        // Display keydown sequence
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

fn check_key_down(
    mut player_command_evt: EventReader<PlayerCommandEvent>,
    mut query: Query<&mut SoundPlayer>,
    mut attack_evt_w: EventWriter<AttackEvent>,
) {
    for e in player_command_evt.read() {
        let action_type = if e.team == 1 {
            ActionType::Player1
        } else {
            ActionType::Player2
        };
        let key = match e.cmd {
            PlayerCommand::Hit1 => 1,
            PlayerCommand::Hit2 => 2,
            PlayerCommand::Hit3 => 3,
            PlayerCommand::Exit => {
                continue;
            }
        };

        for mut player in &mut query {
            if player.action.action_type == action_type {
                let allowed_error = player.interval / 4;

                if player.timer.remaining() > allowed_error {
                    log::debug!(diff = player.timer.remaining().as_secs_f32(), "wrong!");
                    player.fail(&mut attack_evt_w);
                    break;
                }

                if player.pressed {
                    log::debug!("Double Press");
                    player.fail(&mut attack_evt_w);
                    break;
                }

                player.pressed = true;
                log::trace!(key = key);

                player.past_key.push(key);
                if (player.action.keys[player.past_key.len() - 1])
                    != *player.past_key.last().unwrap()
                {
                    log::trace!("wrong combo");
                    player.fail(&mut attack_evt_w);
                }

                if player.action.keys == player.past_key {
                    player.past_key.clear();
                    SoundPlayer::do_action(&player.action.action_type, &mut attack_evt_w);
                    player.reroll();
                }

                break;
            }
        }
    }
}

fn start_sound_player(query: Query<&SoundPlayer>, mut evt_w: EventWriter<SoundPlayerStart>) {
    for sound_player in &query {
        evt_w.send(SoundPlayerStart(sound_player.action.action_type));
    }
}

fn player_hit_sound_system(
    mut player_command_evt: EventReader<PlayerCommandEvent>,
    a: Res<ASound>,
    w: Res<WSound>,
    d: Res<DSound>,
    mut commands: Commands,
) {
    let hit_sound_settings = PlaybackSettings {
        mode: PlaybackMode::Despawn,
        // TODO: custom volume
        volume: Volume::new_relative(2.5),
        ..Default::default()
    };

    for e in player_command_evt.read() {
        match e.cmd {
            PlayerCommand::Hit1 => {
                commands.spawn(AudioBundle {
                    source: a.0.clone(),
                    settings: hit_sound_settings,
                });
            }
            PlayerCommand::Hit2 => {
                commands.spawn(AudioBundle {
                    source: w.0.clone(),
                    settings: hit_sound_settings,
                });
            }
            PlayerCommand::Hit3 => {
                commands.spawn(AudioBundle {
                    source: d.0.clone(),
                    settings: hit_sound_settings,
                });
            }
            PlayerCommand::Exit => {}
        }
    }
}

#[derive(Debug, Component)]
pub struct BeatTimer(pub Timer);

#[derive(Debug, Component)]
pub struct MoveBeat {
    pub from: Vec2,
    pub to: Vec2,
    pub duration: Duration,
}

fn produce_beat_system(mut query: Query<(&SoundPlayer, &mut BeatTimer)>, mut commands: Commands) {
    for (sound_player, mut beat_timer) in &mut query {
        if sound_player.timer.just_finished() {
            let duration = sound_player.timer.remaining();
            match sound_player.action.action_type {
                ActionType::Player1 => {
                    log::trace!("beat p1");
                    commands.spawn(MoveBeat {
                        from: BEAT_START,
                        to: BEAT_END_P1,
                        duration,
                    });
                }
                ActionType::Player2 => {
                    log::trace!("beat p2");
                    commands.spawn(MoveBeat {
                        from: BEAT_START,
                        to: BEAT_END_P2,
                        duration,
                    });
                }
            }
        }
    }
}

fn produce_beat_on_player_start(mut evt: EventReader<SoundPlayerStart>, mut commands: Commands) {
    // FIXME: hard-coded
    let duration = Duration::from_millis(1000);
    for e in evt.read() {
        match e.0 {
            ActionType::Player1 => {
                commands.spawn(MoveBeat {
                    from: BEAT_START,
                    to: BEAT_END_P1,
                    duration,
                });
            }
            ActionType::Player2 => {
                commands.spawn(MoveBeat {
                    from: BEAT_START,
                    to: BEAT_END_P2,
                    duration,
                });
            }
        }
    }
}

fn move_beat_system(
    mut commands: Commands,
    query: Query<(Entity, &MoveBeat)>,
    asset_server: Res<AssetServer>,
) {
    for (ent, mb) in &query {
        let MoveBeat { from, to, duration } = *mb;
        let z = 30.;
        let from = Vec3::new(from.x, from.y, z);
        let to = Vec3::new(to.x, to.y, z);
        let tween = Tween::new(
            EaseMethod::Linear,
            duration,
            TransformPositionLens {
                start: from,
                end: to,
            },
        );

        let img = asset_server.load("images/ui/game/A.png");
        commands.spawn((
            SpriteBundle {
                texture: img,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50., 50.)),
                    ..default()
                },
                transform: Transform {
                    translation: from,
                    ..Default::default()
                },
                ..default()
            },
            Animator::new(tween),
        ));
        commands.entity(ent).despawn();
    }
}
