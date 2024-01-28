use crate::ringcon::RingConEvent;
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

#[derive(Debug)]
pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCommandEvent>().add_systems(
            Update,
            (
                check_keyboard_input,
                // debug_player_command,
                #[cfg(target_os = "windows")]
                check_ringcon_input,
            ),
        );
    }
}

#[derive(Debug)]
pub enum PlayerCommand {
    Hit1,
    Hit2,
    Hit3,
    Exit,
}

#[derive(Debug, Event)]
pub struct PlayerCommandEvent {
    pub team: i32,
    pub cmd: PlayerCommand,
}

fn check_keyboard_input(
    mut kbd_evt: EventReader<KeyboardInput>,
    mut exit_evt_w: EventWriter<AppExit>,
    mut player_cmd_evt_w: EventWriter<PlayerCommandEvent>,
) {
    for e in kbd_evt.read() {
        if e.state != ButtonState::Pressed {
            continue;
        }

        match e.key_code {
            Some(KeyCode::Escape) => {
                exit_evt_w.send(AppExit);
            }
            Some(KeyCode::A) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit1,
                });
            }
            Some(KeyCode::W) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit2,
                });
            }
            Some(KeyCode::D) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit3,
                });
            }
            Some(KeyCode::G) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 2,
                    cmd: PlayerCommand::Hit1,
                });
            }
            Some(KeyCode::Y) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 2,
                    cmd: PlayerCommand::Hit2,
                });
            }
            Some(KeyCode::J) => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 2,
                    cmd: PlayerCommand::Hit3,
                });
            }
            _ => {}
        }
    }
}

fn check_ringcon_input(
    mut ringcon_evt: EventReader<RingConEvent>,
    mut player_cmd_evt_w: EventWriter<PlayerCommandEvent>,
) {
    for e in ringcon_evt.read() {
        match e {
            RingConEvent::Push => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit1,
                });
            }
            RingConEvent::Pull => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit2,
                });
            }
            RingConEvent::Squat => {
                player_cmd_evt_w.send(PlayerCommandEvent {
                    team: 1,
                    cmd: PlayerCommand::Hit3,
                });
            }
        }
    }
}

fn debug_player_command(mut evt: EventReader<PlayerCommandEvent>) {
    for e in evt.read() {
        eprintln!("{e:?}");
    }
}
