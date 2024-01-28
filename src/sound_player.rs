use crate::config::ImageKey;
use crate::{AppState, AttackEvent};
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseFunction, EaseMethod, Tween};
use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec;

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
            action_type: action_type,
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

        if ((self.action_start_time).abs_diff(since_the_epoch.as_millis()) < interval) {
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

        if vec_compare(&self.action.keys, &self.past_key) {
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
        let start = SystemTime::now();

        if (time - self.start_time) / self.interval <= self.last_step {
            return false;
        }

        self.last_step = (time - self.start_time) / self.interval;

        self.pressed = false;

        return true;
    }
}

#[derive(Debug)]
pub struct SoundSystemPlugin;

impl Plugin for SoundSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundPlayerStart>().add_systems(
            Update,
            (
                produce_beat_system,
                move_beat_system,
                produce_beat_on_player_start,
            )
                .run_if(in_state(AppState::InGame)),
        );
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
