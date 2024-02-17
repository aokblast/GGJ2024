use bevy::app::{App, Plugin, Startup};
use bevy::log;
use bevy::prelude::{Event, EventWriter, Res, ResMut, Resource, Update};
use bevy::time::{Time, Timer, TimerMode};
use dlopen2::wrapper::{Container, WrapperApi};
use std::time::Duration;

#[derive(Debug)]
pub struct RingConPlugin;

const PUSHING_THRESHOLD: i32 = 7;
const PULLING_THRESHOLD: i32 = 2;

const SQUATTING_TIME: u64 = 500;
const SQUATTING_THRESHOLD: f64 = 0.5;

#[derive(Copy, Clone, Eq, PartialEq, Default)]
enum SquattingStates {
    #[default]
    No,
    Doing,
    Done,
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
    Push,
    Pull,
    Squat,
}

impl RingConRS {
    fn new() -> Self {
        let dll_path = "./ringcon_driver.dll";
        Self {
            container: unsafe { Container::load(dll_path) }.unwrap(),
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
            api.container.poll_ringcon(&mut res as *mut PullVal);
        }

        log::trace!("{}", res.push_val);

        let detected_key = {
            if res.push_val >= PUSHING_THRESHOLD {
                Some(RingConEvent::Push)
            } else if res.push_val <= PULLING_THRESHOLD {
                Some(RingConEvent::Pull)
            } else {
                None
            }
        };

        if let Some(key) = detected_key {
            match api.ring_stat {
                Some(key2) if key2 != key => event.send(key),
                None => event.send(key),
                _ => {}
            }
        }

        api.ring_stat = detected_key;
    }

    if api.squat_rs.stat == SquattingStates::Doing {
        if res.squatting {
            api.squat_rs.sq += 1;
        }

        api.squat_rs.nsq += 1;
    }

    api.squat_timer.tick(time.delta());
    if api.squat_timer.finished() {
        let mut stat = api.squat_rs.stat;

        if stat == SquattingStates::Doing {
            if (api.squat_rs.sq as f64 / api.squat_rs.nsq as f64) >= SQUATTING_THRESHOLD {
                stat = SquattingStates::Done;
            } else {
                stat = SquattingStates::No;
            }
        }

        if stat == SquattingStates::No {
            if res.squatting {
                stat = SquattingStates::Doing;
                api.squat_rs.sq = 0;
                api.squat_rs.nsq = 0;
            }
        } else if stat == SquattingStates::Done {
            event.send(RingConEvent::Squat);
            stat = SquattingStates::No;
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
