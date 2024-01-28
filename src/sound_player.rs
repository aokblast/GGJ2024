use bevy::prelude::*;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;

use crate::{main, score_system, AttackEvent, ComboNumber, CounterNumber};

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
    has_started: bool,
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
