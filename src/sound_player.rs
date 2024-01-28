use crate::config::ImageKey;
use crate::{AppState, AttackEvent};
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseFunction, Tween};
use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec;

#[derive(PartialEq, Eq)]
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

#[derive(Component)]
pub struct SoundPlayer {
    pub action: Action,
    interval: u128,
    start_time: u128,
    action_start_time: u128,
    pub past_key: Vec<i32>,
    pub has_started: bool,
    last_step: u128,
    pub sound_id: Entity,
    pub goal_text_id: Entity,
    pub past_text_id: Entity,
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
            past_key: vec![],
            has_started: false,
            last_step: 0,
            sound_id,
            goal_text_id,
            past_text_id,
        }
    }

    pub fn start(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        self.start_time = since_the_epoch.as_millis();
        self.reroll();
        self.has_started = true;
    }

    fn reroll(&mut self) {
        self.action.keys.clear();
        let len = rand::thread_rng().gen_range(1..6);
        for _ in 0..len {
            self.action.keys.push(rand::thread_rng().gen_range(1..4));
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

    pub fn key_down(&mut self, key: i32, evt_w: &mut EventWriter<AttackEvent>) {
        if !self.has_started {
            return;
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut remain = (since_the_epoch.as_millis() - self.start_time) % self.interval;

        if remain > self.interval / 2 {
            remain = self.interval - remain;
        }

        if remain > self.interval / 4 {
            println!("wrong");
            self.fail(evt_w);
            self.past_key.clear();
            return;
        }

        if self.past_key.len() == 0 {
            self.action_start_time = since_the_epoch.as_millis()
        }

        let passing_keys = (since_the_epoch.as_millis() - self.action_start_time
            + self.interval / 2)
            / self.interval;

        let missing_key = 1 + passing_keys as usize - self.past_key.len();

        if missing_key < 1 {
            println!("too fast");
            self.fail(evt_w);
            self.past_key.clear();
            return;
        }

        if missing_key > 1 {
            println!("too slow");
            self.fail(evt_w);
            self.past_key.clear();
            return;
        }

        println!("{}", key);

        self.past_key.push(key);

        if vec_compare(&self.action.keys, &self.past_key) {
            self.past_key.clear();
            Self::do_action(&self.action.action_type, evt_w);
            self.reroll();
        } else if self.past_key.len() >= self.action.keys.len() {
            println!("wrong combo");
            self.fail(evt_w);
            self.past_key.clear();
        }
    }

    pub fn end(&mut self) {
        self.has_started = false;
    }

    pub fn update(&mut self) -> bool {
        if !self.has_started {
            return false;
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        if (since_the_epoch.as_millis() - self.start_time) / self.interval <= self.last_step {
            return false;
        }
        self.last_step = (since_the_epoch.as_millis() - self.start_time) / self.interval;

        return true;
    }
}

#[derive(Debug)]
pub struct SoundSystemPlugin;

impl Plugin for SoundSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (produce_beat_system, move_beat_system).run_if(in_state(AppState::InGame)),
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

const BEAT_START: Vec2 = Vec2::new(0., 400.);
const BEAT_END_P1: Vec2 = Vec2::new(-500., 400.);
const BEAT_END_P2: Vec2 = Vec2::new(500., 400.);

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

fn move_beat_system(
    mut commands: Commands,
    query: Query<(Entity, &MoveBeat)>,
    asset_server: Res<AssetServer>,
) {
    for (ent, mb) in &query {
        let MoveBeat { from, to, duration } = *mb;
        let z = -20.;
        let from = Vec3::new(from.x, from.y, z);
        let to = Vec3::new(to.x, to.y, z);
        let tween = Tween::new(
            EaseFunction::BackInOut,
            duration,
            TransformPositionLens {
                start: from,
                end: to,
            },
        );

        let img = asset_server.load(format!("images/{}", ImageKey::GenShinStart));
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50., 50.)),
                    ..Default::default()
                },
                texture: img,
                transform: Transform {
                    translation: from,
                    ..Default::default()
                },
                ..Default::default()
            },
            Animator::new(tween),
        ));
        commands.entity(ent).despawn();
    }
}
