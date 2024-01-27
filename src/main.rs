mod sound_player;

use bevy::{input::{keyboard::KeyboardInput, ButtonState}, prelude::*};
use sound_player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, phah)
        .run();
}

fn setup(mut commands: Commands) {
    let mut sound_player: SoundPlayer = SoundPlayer::new(5000);
    sound_player.add_action(Action::new(vec![1, 2, 2], ActionType::Attack));
    sound_player.start();

    commands.insert_resource(sound_player);
}

fn phah(mut events: EventReader<KeyboardInput>, mut sound_player: ResMut<SoundPlayer>) {
    for event in events.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                Some(KeyCode::A) => {
                    sound_player.key_down(1);
                }
                Some(KeyCode::S) => {
                    sound_player.key_down(2);
                }
                _ => continue,
            }
        }
    }
}
