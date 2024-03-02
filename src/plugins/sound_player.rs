use super::Team;
use crate::plugins::input::{PlayerCommand, PlayerCommandEvent};
use crate::plugins::score::AttackEvent;
use crate::AppState;
use bevy::audio::{PlaybackMode, Volume};
use bevy::time::Stopwatch;
use bevy::{log, prelude::*};
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseMethod, Tween};
use rand::{thread_rng, Rng};
use std::time::Duration;

const BEAT_START: Vec2 = Vec2::new(0., -450.);
const BEAT_END_P1: Vec2 = Vec2::new(-700., -450.);
const BEAT_END_P2: Vec2 = Vec2::new(700., -450.);
const BEAT_RING_OFFSET: f32 = 100.;

#[derive(Debug)]
pub struct SoundSystemPlugin;

impl Plugin for SoundSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_sound_system)
            .add_systems(OnEnter(AppState::InGame), produce_beat_one_shot)
            .add_systems(
                Update,
                (
                    sound_timer,
                    check_key_down,
                    produce_beat_system,
                    move_beat_system,
                    player_hit_sound_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Debug, Component)]
struct BeatControl {
    stopwatch: Stopwatch,
    time_delta: Duration,
    last_gen: Duration,
    allowed_error: Duration,
}

#[derive(Debug)]
enum HitResult {
    Perfect,
    Good,
    Ok,
    Miss,
}

fn check_hit_result(error: Duration, delta: Duration) -> Option<HitResult> {
    if delta > error {
        return None;
    }

    if delta > error * 3 / 4 {
        return Some(HitResult::Miss);
    }

    if delta > error / 2 {
        return Some(HitResult::Ok);
    }

    if delta > error / 3 {
        return Some(HitResult::Good);
    }

    Some(HitResult::Perfect)
}

#[derive(Debug, Component)]
struct Beat {
    hit_point: Duration,
    key: i32,
}

#[derive(Component)]
pub struct Sound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct WSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct ASound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct DSound(pub Handle<AudioSource>);

fn setup_sound_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ASound(asset_server.load("sounds/A.ogg")));
    commands.insert_resource(WSound(asset_server.load("sounds/W.ogg")));
    commands.insert_resource(DSound(asset_server.load("sounds/D.ogg")));

    let ring_img = asset_server.load("images/ui/game/white.png");
    commands.spawn(SpriteBundle {
        texture: ring_img.clone(),
        transform: Transform {
            translation: BEAT_END_P1.extend(10.), // + Vec3::new(BEAT_RING_OFFSET, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: ring_img,
        transform: Transform {
            translation: BEAT_END_P2.extend(10.), // - Vec3::new(BEAT_RING_OFFSET, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn(BeatControl {
        stopwatch: Stopwatch::new(),
        time_delta: Duration::from_secs(5),
        last_gen: Duration::default(),
        allowed_error: Duration::from_secs_f32(0.25),
    });
}

fn sound_timer(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut beat_ctl_query: Query<&mut BeatControl>,
    beat_query: Query<(Entity, &Beat), Without<Team>>,
) {
    let mut beat_ctl = beat_ctl_query.get_single_mut().unwrap();

    beat_ctl.stopwatch.tick(time.delta());
    let elapsed = beat_ctl.stopwatch.elapsed();
    let sound = asset_server.load("sounds/gong.ogg");
    for (ent, beat) in &beat_query {
        // FIXME: hard-coded
        if beat
            .hit_point
            .checked_sub(elapsed)
            .unwrap_or_else(|| elapsed.checked_sub(beat.hit_point).unwrap())
            < Duration::from_millis(16)
        {
            commands.spawn(AudioBundle {
                source: sound.clone(),
                // auto-despawn the entity when playback finishes
                settings: PlaybackSettings::DESPAWN,
            });
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn check_key_down(
    mut player_command_evt: EventReader<PlayerCommandEvent>,
    beat_query: Query<(Entity, &Beat, &Team), With<MoveBeat>>,
    mut attack_evt_w: EventWriter<AttackEvent>,
    beat_ctl_query: Query<&BeatControl>,
    mut commands: Commands,
) {
    let beat_ctl = beat_ctl_query.get_single().unwrap();

    for e in player_command_evt.read() {
        let e_team = Team(e.team);
        let key = match e.cmd {
            PlayerCommand::Hit1 => 1,
            PlayerCommand::Hit2 => 2,
            PlayerCommand::Hit3 => 3,
            PlayerCommand::Exit => {
                continue;
            }
        };

        for (b_ent, beat, b_team) in &beat_query {
            if *b_team == e_team {
                let allowed_error = beat_ctl.allowed_error;
                let delta =
                    if let Some(diff) = beat.hit_point.checked_sub(beat_ctl.stopwatch.elapsed()) {
                        diff
                    } else {
                        beat_ctl.stopwatch.elapsed() - beat.hit_point
                    };
                let Some(hit_result) = check_hit_result(allowed_error, delta) else {
                    continue;
                };

                match hit_result {
                    HitResult::Miss => {
                        log::debug!(diff = delta.as_secs_f32(), "miss");
                        attack_evt_w.send(AttackEvent(b_team.0, false));
                    }
                    _ => {
                        if key != beat.key {
                            log::trace!("wrong key");
                            attack_evt_w.send(AttackEvent(b_team.0, false));
                        } else {
                            log::info!(team = b_team.0, "player attack");
                            attack_evt_w.send(AttackEvent(b_team.0, true));
                        }
                    }
                }
                commands.entity(b_ent).despawn_recursive();

                break;
            }
        }
    }

    let elapsed = beat_ctl.stopwatch.elapsed();
    for (b_ent, beat, _) in &beat_query {
        if elapsed > beat.hit_point
            && matches!(
                check_hit_result(beat_ctl.allowed_error, elapsed - beat.hit_point),
                Some(HitResult::Miss) | None
            )
        {
            commands.entity(b_ent).despawn_recursive();
        }
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
pub struct MoveBeat {
    pub from: Vec2,
    pub to: Vec2,
    pub duration: Duration,
}

fn produce_beat_system(mut beat_ctl_query: Query<&mut BeatControl>, mut commands: Commands) {
    let mut beat_ctl = beat_ctl_query.get_single_mut().unwrap();

    let gen_delta = Duration::from_secs(1);
    loop {
        let gen = beat_ctl.last_gen + gen_delta;
        if gen > beat_ctl.stopwatch.elapsed() + beat_ctl.time_delta {
            break;
        }

        let mut rng = thread_rng();
        let key = rng.gen_range(1..=3);
        commands.spawn((
            Beat {
                hit_point: gen,
                key,
            },
            Team(1),
        ));
        commands.spawn((
            Beat {
                hit_point: gen,
                key,
            },
            Team(2),
        ));
        commands.spawn(Beat {
            hit_point: gen,
            key: -1,
        });
        beat_ctl.last_gen = gen;
    }
}

fn produce_beat_one_shot(mut query: Query<&mut BeatControl>, mut commands: Commands) {
    // let mut beat_ctl = query.get_single_mut().unwrap();
    // let hit_point = beat_ctl.stopwatch.elapsed() + Duration::from_secs(1);
    // commands.spawn((Beat { hit_point, key: 1 }, Team(1)));
    // commands.spawn((Beat { hit_point, key: 1 }, Team(2)));
}

fn move_beat_system(
    mut commands: Commands,
    query: Query<(Entity, &Beat, &Team), Without<MoveBeat>>,
    beat_ctl_query: Query<&BeatControl>,
    asset_server: Res<AssetServer>,
) {
    let move_duration = Duration::from_secs(2);
    let beat_ctl = beat_ctl_query.get_single().unwrap();

    for (ent, beat, team) in &query {
        let remain_time = beat.hit_point - beat_ctl.stopwatch.elapsed();
        if remain_time > move_duration {
            continue;
        }

        let (from, to) = match team.0 {
            1 => (BEAT_START, BEAT_END_P1),
            2 => (BEAT_START, BEAT_END_P2),
            _ => panic!(),
        };
        let z = 30.;
        let from = Vec3::new(from.x, from.y, z);
        let to = Vec3::new(to.x, to.y, z);
        let tween = Tween::new(
            EaseMethod::Linear,
            remain_time,
            TransformPositionLens {
                start: from,
                end: to,
            },
        );

        // TODO: Res
        let img = match (team.0, beat.key) {
            (1, 1) => asset_server.load("images/ui/game/A.png"),
            (1, 2) => asset_server.load("images/ui/game/W.png"),
            (1, 3) => asset_server.load("images/ui/game/D.png"),
            (2, 1) => asset_server.load("images/ui/game/G.png"),
            (2, 2) => asset_server.load("images/ui/game/Y.png"),
            (2, 3) => asset_server.load("images/ui/game/J.png"),
            _ => unreachable!(),
        };
        commands.entity(ent).insert((
            SpriteBundle {
                texture: img,
                transform: Transform {
                    translation: from,
                    ..Default::default()
                },
                ..default()
            },
            Animator::new(tween),
            MoveBeat {
                from: from.truncate(),
                to: to.truncate(),
                duration: remain_time,
            },
        ));
    }
}
