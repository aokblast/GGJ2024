use crate::config::ImageKey;
use crate::plugins::input::{PlayerCommand, PlayerCommandEvent};
use crate::{AppState, AttackEvent};
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseMethod, Tween};
use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec;

#[derive(Debug)]
pub struct SoundSystemPlugin;

impl Plugin for SoundSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundPlayerStart>()
            .add_systems(Startup, setup_sound_system)
            .add_systems(OnEnter(AppState::InGame), start_sound_player)
            .add_systems(
                Update,
                (
                    produce_beat_system,
                    move_beat_system,
                    produce_beat_on_player_start,
                    sound_timer,
                    check_key_down,
                    player_hit_sound_system,
                )
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
    interval: u128,
    start_time: u128,
    action_start_time: u128,
    action_last_time: u128,
    pub past_key: Vec<i32>,
    pub has_started: bool,
    last_step: u128,
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

fn vec_compare(va: &[i32], vb: &[i32]) -> bool {
    (va.len() == vb.len()) &&  // zip stops at the shortest
     va.iter()
       .zip(vb)
       .all(|(a,b)| a == b)
}

impl SoundPlayer {
    pub fn new(
        interval: u128,
        action_type: ActionType,
        sound_id: Entity,
        goal_text_id: Entity,
        past_text_id: Entity,
    ) -> Self {
        Self {
            action: Action::new(action_type),
            interval,
            start_time: 0,
            action_start_time: 0,
            action_last_time: 0,
            past_key: vec![],
            has_started: false,
            last_step: 0,
            sound_id,
            goal_text_id,
            past_text_id,
            pressed: false,
        }
    }

    pub fn start(&mut self, evt_w: &mut EventWriter<SoundPlayerStart>) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        self.start_time = since_the_epoch.as_millis();
        self.reroll();
        self.has_started = true;

        evt_w.send(SoundPlayerStart(self.action.action_type));
    }

    fn reroll(&mut self) {
        self.action.keys.clear();
        let len = rand::thread_rng().gen_range(1..6);
        for _ in 0..len {
            self.action.keys.push(rand::thread_rng().gen_range(1..3));
        }
    }

    fn do_action(action_type: &ActionType, evt_w: &mut EventWriter<AttackEvent>) {
        match action_type {
            ActionType::Player1 => {
                println!("Player1");
                evt_w.send(AttackEvent(1, true));
            }
            ActionType::Player2 => {
                println!("Player2");
                evt_w.send(AttackEvent(2, true));
            }
        }
    }

    fn fail(&self, evt_w: &mut EventWriter<AttackEvent>) {
        evt_w.send(AttackEvent(
            if self.action.action_type == ActionType::Player1 {
                1
            } else {
                2
            },
            false,
        ));
    }

    pub fn key_down(&mut self, key: i32, evt_w: &mut EventWriter<AttackEvent>, is_ringcon: bool) {
        if !self.has_started {
            return;
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut interval = self.interval / 4;

        if is_ringcon {
            interval *= 2;
        }

        if self.action_start_time.abs_diff(since_the_epoch.as_millis()) < interval {
            println!("wrong");
            self.fail(evt_w);
            self.past_key.clear();
            return;
        }

        if self.pressed {
            println!("Double Press");
            self.fail(evt_w);
            self.past_key.clear();
            return;
        }

        self.pressed = true;

        println!("{}", key);

        self.past_key.push(key);

        if (self.action.keys[self.past_key.len() - 1]) != *self.past_key.last().unwrap() {
            println!("wrong combo");
            self.fail(evt_w);
            self.past_key.clear();
        }

        if self.action.keys == self.past_key {
            self.past_key.clear();
            Self::do_action(&self.action.action_type, evt_w);
            self.reroll();
        }
    }

    pub fn end(&mut self) {
        self.has_started = false;
    }

    pub fn update(&mut self, time: u128) -> bool {
        if !self.has_started {
            return false;
        }
        self.action_last_time = self.action_start_time;
        self.action_start_time = time;
        let _start = SystemTime::now();

        if (time - self.start_time) / self.interval <= self.last_step {
            return false;
        }

        self.last_step = (time - self.start_time) / self.interval;

        self.pressed = false;

        return true;
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

    commands.insert_resource(ASound(asset_server.load("sounds/A.ogg")));
    commands.insert_resource(WSound(asset_server.load("sounds/W.ogg")));
    commands.insert_resource(DSound(asset_server.load("sounds/D.ogg")));
}

fn sound_timer(
    mut commands: Commands,
    mut query: Query<&mut SoundPlayer>,
    mut text_query: Query<&mut Text>,
    sound_query: Query<&Sound>,
) {
    for mut sound_player in &mut query {
        if sound_player.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
                + 1000,
        ) {
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
                player.key_down(key, &mut attack_evt_w, false);
                break;
            }
        }
    }
}

fn start_sound_player(
    mut query: Query<&mut SoundPlayer>,
    mut sound_start_evt_w: EventWriter<SoundPlayerStart>,
) {
    for mut sound_player in &mut query {
        sound_player.start(&mut sound_start_evt_w);
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

const BEAT_START: Vec2 = Vec2::new(0., -400.);
const BEAT_END_P1: Vec2 = Vec2::new(-500., -400.);
const BEAT_END_P2: Vec2 = Vec2::new(500., -400.);

fn produce_beat_system(
    mut query: Query<(&SoundPlayer, &mut BeatTimer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (sound_player, mut beat_timer) in &mut query {
        if !sound_player.has_started {
            continue;
        }

        beat_timer.0.tick(time.delta());
        if beat_timer.0.just_finished() {
            let duration = Duration::from_millis(sound_player.interval as u64);
            match sound_player.action.action_type {
                ActionType::Player1 => {
                    eprintln!("beat p1");
                    commands.spawn(MoveBeat {
                        from: BEAT_START,
                        to: BEAT_END_P1,
                        duration,
                    });
                }
                ActionType::Player2 => {
                    eprintln!("beat p2");
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

        let img = asset_server.load(format!("images/{}", ImageKey::GenShinStart));
        commands.spawn((
            SpriteBundle {
                texture: img,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50., 50.)),
                    ..default()
                },
                ..default()
            },
            Animator::new(tween),
        ));
        commands.entity(ent).despawn();
    }
}
