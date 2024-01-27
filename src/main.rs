mod sound_player;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use sound_player::*;

#[derive(Resource)]
struct StepSound(Handle<AudioSource>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, phah)
        .add_systems(Update, sound_timer)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut sound_player: SoundPlayer = SoundPlayer::new(1000);
    sound_player.add_action(Action::new(vec![1, 2, 2], ActionType::Attack));

    commands.insert_resource(sound_player);

    let ball_collision_sound = asset_server.load("sounds/gong.ogg");
    commands.insert_resource(StepSound(ball_collision_sound));
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
                Some(KeyCode::D) => {
                    println!("start");
                    sound_player.start();
                }
                _ => continue,
            }
        }
    }
}

fn sound_timer(
    mut sound_player: ResMut<SoundPlayer>,
    mut commands: Commands,
    sound: Res<StepSound>,
) {
    if sound_player.update() {
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
    }
}
