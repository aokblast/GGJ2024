use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;

use crate::{main, score_system, AttackEvent, ComboNumber, CounterNumber};

pub enum ActionType {
    Attack,
    Defence,
    Shoot,
}

pub struct Action {
    keys: Vec<i32>,
    action_type: ActionType,
}

#[derive(Resource)]
pub struct StepSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct SoundPlayer {
    actions: Vec<Action>,
    interval: u128,
    start_time: u128,
    action_start_time: u128,
    pub past_key: Vec<i32>,
    has_started: bool,
    last_step: u128,
    max_key: usize,
}

impl Action {
    pub fn new(keys: Vec<i32>, action_type: ActionType) -> Self {
        Self {
            keys: keys,
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
    pub fn new(interval: u128) -> Self {
        Self {
            actions: vec![],
            interval,
            start_time: 0,
            action_start_time: 0,
            past_key: vec![],
            has_started: false,
            last_step: 0,
            max_key: 0,
        }
    }

    pub fn add_action(&mut self, action: Action) {
        let l = action.keys.len();
        self.actions.push(action);
        if l > self.max_key {
            self.max_key = l;
        }
    }

    pub fn start(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        self.start_time = since_the_epoch.as_millis();
        self.has_started = true;
    }

    fn do_action(&self, action_type: &ActionType, evt_w: &mut EventWriter<AttackEvent>) {
        match action_type {
            ActionType::Attack => {
                println!("Attack");
                evt_w.send(AttackEvent(1, true));
            }
            ActionType::Defence => {
                println!("Defence");
            }
            ActionType::Shoot => {
                println!("Shoot");
            }
        }
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
            self.past_key.clear();
            return;
        }

        if missing_key > 1 {
            println!("too slow");
            self.past_key.clear();
            return;
        }

        println!("{}", key);

        self.past_key.push(key);
        let match_action = self
            .actions
            .iter()
            .filter(|a| vec_compare(&a.keys, &self.past_key))
            .next();

        if let Some(a) = match_action {
            self.past_key.clear();
            self.do_action(&a.action_type, evt_w);
        } else if self.past_key.len() >= self.max_key {
            println!("wrong combo");
            self.past_key.clear();
        }
    }

    pub fn end(&mut self) {
        self.has_started = false;
    }

    pub fn update(&mut self, mut commands: Commands, sound: Res<StepSound>) -> bool {
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

        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });

        // if self.action_start_time != 0 {
        //     let passing_keys = (since_the_epoch.as_millis() - self.action_start_time
        //         + self.interval / 2)
        //         / self.interval;

        //     let missing_key = 1 + passing_keys as usize - self.past_key.len();

        //     for _ in 1..missing_key {
        //         self.past_key.push(0);
        //     }
        //     for k in &self.past_key {
        //         print!("{}, ", k);
        //     }
        //     if self.past_key.len() >= 3 {
        //         println!("wrong combo");
        //         self.past_key.clear();
        //         self.action_start_time = 0;
        //     }

        //     println!("");
        // }

        return true;
    }
}
