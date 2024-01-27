mod sound_player;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use sound_player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, phah)
        .add_systems(Update, sound_timer)
        .insert_resource(CounterNumber { score: 123 })
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (counter_system, (counter_update_system, text_color_system)).chain(),
        )
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

fn sound_timer(mut sound_player: ResMut<SoundPlayer>, commands: Commands, sound: Res<StepSound>) {
    sound_player.update(commands, sound);
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    select,
    InGame,
}

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
struct Colortext;

const COUNTER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const SCOREBOARD_FONT_SIZE: f32 = 40.0;

#[derive(Component)]
struct CounterText;

#[derive(Resource)]
struct CounterNumber {
    score: usize,
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                // disable clearing completely (pixels stay as they are)
                // (preserves output from previous frame or camera/pass)
                clear_color: ClearColorConfig::Custom(Color::rgb(0.5, 0.2, 0.2)),
            },
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
        MyCameraMarker,
    ));
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "PaTaPon!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                ..default()
            },
        ) // Set the justification of the Text
        //.with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        Colortext,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: COUNTER_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: COUNTER_COLOR,
                ..default()
            }),
        ]),
        CounterText,
    ));
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<Colortext>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}
fn counter_system(mut counterboard: ResMut<CounterNumber>) {
    if true {
        //attack deals demage
        counterboard.score += 1;
    }
}

fn counter_update_system(
    counter: Res<CounterNumber>,
    mut query: Query<&mut Text, With<CounterText>>,
) {
    for mut text in &mut query {
        text.sections[1].value = counter.score.to_string();
    }
}
// fn text_update_system(scoreboard: Res<Scoreboard>,mut query: Query<&mut Text, With<CounterText>>) {
//     for mut text in &mut query {
//         let mut value = query.single_mut();
//         // Update counter to  value of the second section
//         text.sections[1].value = scoreboard.score.to_string()
//     }
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)] //+, Reflect, Serialize, Deserialize#[reflect(Serialize, Deserialize)]
pub enum JustifyText {
    /// Leftmost character is immediately to the right of the render position.
    /// Bounds start from the render position and advance rightwards.
    #[default]
    Left,
    /// Leftmost & rightmost characters are equidistant to the render position.
    /// Bounds start from the render position and advance equally left & right.
    Center,
    /// Rightmost character is immediately to the left of the render position.
    /// Bounds start from the render position and advance leftwards.
    Right,
}
