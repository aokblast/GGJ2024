use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;

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
pub struct SoundPlayer {
    actions: Vec<Action>,
    interval: u128,
    start_time: u128,
    past_key: Vec<i32>,
    has_started: bool,
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
            interval: interval,
            start_time: 0,
            past_key: vec![],
            has_started: false,
        }
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action)
    }

    pub fn start(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        self.start_time = since_the_epoch.as_millis();
        self.has_started = true;
    }

    fn do_action(&self, action_type: &ActionType) {
        match action_type {
            ActionType::Attack => {
                println!("Attack");
            }
            ActionType::Defence => {
                println!("Defence");
            }
            ActionType::Shoot => {
                println!("Shoot");
            }
        }
    }

    pub fn key_down(&mut self, key: i32) {
        if !self.has_started {
            return;
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut remain = since_the_epoch.as_millis() - self.start_time;
        self.start_time = since_the_epoch.as_millis();

        if remain < self.interval {
            remain = self.interval - remain;
        } else {
            remain = remain - self.interval;
        }

        if remain > self.interval / 8 {
            println!("wrong");
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
            self.do_action(&a.action_type);
        } else if self.past_key.len() >= 3 {
            println!("wrong combo");
            self.past_key.clear();
        }
    }

    pub fn end(&mut self) {
        self.has_started = false;
    }
}