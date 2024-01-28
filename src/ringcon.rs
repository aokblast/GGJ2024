use crate::ringcon::RingConEvent::SD;
use crate::ringcon::SquattingStates::{DOING, DONE, NO};
use crate::AppState::InGame;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Entity, Event, EventWriter, OnEnter, Res, ResMut, Resource, Update};
use bevy::time::{Time, Timer, TimerMode};
use dlopen2::wrapper::{Container, WrapperApi};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct RingConPlugin;

const PUSHING_THRESHOLD: i32 = 7;
const PULLING_THRESHOLD: i32 = 2;

const SQUATTING_TIME: u64 = 500;
const SQUATTING_THRESHOLD: f64 = 0.5;

#[derive(Copy, Clone, Eq, PartialEq)]
enum SquattingStates {
    NO,
    DOING,
    DONE,
}

impl Default for SquattingStates {
    fn default() -> Self {
        Self::NO
    }
}

#[derive(WrapperApi)]
pub struct RingConApi {
    ringcon_init: unsafe extern "C" fn(),
    poll_ringcon: unsafe extern "C" fn(pull_val: *mut PullVal),
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct PullVal {
    pub running: bool,
    pub squatting: bool,
    pub push_val: i32,
}
#[derive(Resource)]
struct RingConRS {
    pub container: Container<RingConApi>,
    pub timer: Timer,
    pub squat_timer: Timer,
    pub squat_rs: SquatRS,
    pub ring_stat: Option<RingConEvent>,
}
#[derive(Default)]
struct SquatRS {
    pub stat: SquattingStates,
    pub sq: i64,
    pub nsq: i64,
}

#[derive(Event, Eq, PartialEq, Copy, Clone)]
pub enum RingConEvent {
    PUSH,
    POLL,
    SD,
}

impl RingConRS {
    fn new() -> Self {
        Self {
            container: unsafe { Container::load("./ringcon_driver.dll") }.unwrap(),
            timer: Timer::new(Duration::from_millis(33), TimerMode::Repeating),
            squat_timer: Timer::new(Duration::from_millis(SQUATTING_TIME), TimerMode::Repeating),
            squat_rs: SquatRS::default(),
            ring_stat: None,
        }
    }
}

fn ringcon_init(api: Res<RingConRS>) {
    unsafe {
        api.container.ringcon_init();
    }
}

fn pull_ringcon_system(
    mut api: ResMut<RingConRS>,
    mut event: EventWriter<RingConEvent>,
    time: Res<Time>,
) {
    let mut res = PullVal::default();

    api.timer.tick(time.delta());
    if api.timer.finished() {
        unsafe {
            api.container
                .poll_ringcon(unsafe { &mut res } as *mut PullVal);
        }

        println!("{}", res.push_val);

        let detected_key = {
            if res.push_val >= PUSHING_THRESHOLD {
                Some(RingConEvent::PUSH)
            } else if res.push_val <= PULLING_THRESHOLD {
                Some(RingConEvent::POLL)
            } else {
                None
            }
        };

        if let Some(key) = detected_key {
            if let Some(key2) = api.ring_stat {
                if key != key2 {
                    event.send(key);
                }
            }

            if api.ring_stat.is_none() {
                event.send(key);
            }
        }

        api.ring_stat = detected_key;
    }

    if api.squat_rs.stat == DOING {
        if res.squatting {
            api.squat_rs.sq += 1;
        }

        api.squat_rs.nsq += 1;
    }

    api.squat_timer.tick(time.delta());
    if api.squat_timer.finished() {
        let mut stat = api.squat_rs.stat;

        if stat == DOING {
            if (api.squat_rs.sq as f64 / api.squat_rs.nsq as f64) >= SQUATTING_THRESHOLD {
                stat = DONE;
            } else {
                stat = NO;
            }
        }

        if stat == SquattingStates::NO {
            if res.squatting {
                stat = SquattingStates::DOING;
                api.squat_rs.sq = 0;
                api.squat_rs.nsq = 0;
            }
        } else if stat == SquattingStates::DONE {
            event.send(SD);
            stat = SquattingStates::NO;
            api.squat_rs.sq = 0;
            api.squat_rs.nsq = 0;
        }

        api.squat_rs.stat = stat;
    }
}

impl Plugin for RingConPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RingConRS::new())
            .add_systems(Startup, ringcon_init)
            .add_systems(Update, pull_ringcon_system)
            .add_event::<RingConEvent>();
    }
}
