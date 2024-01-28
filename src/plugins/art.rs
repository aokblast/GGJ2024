use crate::GenEvent;
use bevy::{prelude::*, sprite::Anchor};
use bevy_tweening::{
    lens::TransformPositionLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween,
};
use rand::prelude::*;
use std::time::Duration;

#[derive(Debug, Component)]
pub struct JumpPeopleImage {
    pub(crate) img_name: String,
    pub(crate) from: Vec2,
    pub(crate) to: Vec2,
}

/// play one-shot SFV
// pub struct PlaySfxEvent;

pub struct ArtPlugin;

#[derive(Debug, Event)]
pub struct PeopleEntity(Entity);

pub fn create_people_system(
    mut commands: Commands,
    query: Query<(Entity, &JumpPeopleImage)>,
    asset_server: Res<AssetServer>,
) {
    for (ent, img) in &query {
        eprint!("jump people {:?}", ent);
        let from = Vec3::new(img.from.x, img.from.y, 10.);
        let to = Vec3::new(img.to.x, img.to.y, 10.);
        let img: Handle<Image> = asset_server.load(img.img_name.to_string());

        let duration = Duration::from_secs_f32(0.5);
        let pos_tween = Tween::new(
            EaseFunction::CircularInOut,
            duration,
            TransformPositionLens {
                start: from,
                end: to,
            },
        )
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
        .with_repeat_count(RepeatCount::Infinite);

        commands.spawn((
            // Spawn a Sprite entity to animate the position of.
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::Center,
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(0., 0., 10.),
                    ..default()
                },
                texture: img,
                ..default()
            },
            // Add an Animator component to control and execute the animation.
            Animator::new(pos_tween),
        ));
        commands.entity(ent).remove::<JumpPeopleImage>();
    }
}

pub fn gen_people(mut commands: Commands, mut evt_r: EventReader<GenEvent>) {
    for e in evt_r.read() {
        let floor = -50.;
        let roof = 50.;
        let che = 500.;
        let lo = 50.;
        let hi = 800.;
        let mut rng = thread_rng();
        let r = rng.gen_range(lo..hi);
        if e.0 == 1 {
            //player 1
            if e.1 == 1 {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/people_1.png".to_string(),
                    from: Vec2 { x: -r, y: floor },
                    to: Vec2 { x: -r, y: roof },
                });
            } else if e.1 == 2 {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/cat_1.png".to_string(),
                    from: Vec2 { x: -r, y: floor },
                    to: Vec2 { x: -r, y: roof },
                });
            } else {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/sedan_chair_1.png".to_string(),
                    from: Vec2 { x: -che, y: floor },
                    to: Vec2 { x: -che, y: roof },
                });
            }
        } else {
            //player2
            if e.1 == 1 {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/people_2.png".to_string(),
                    from: Vec2 { x: r, y: floor },
                    to: Vec2 { x: r, y: roof },
                });
            } else if e.1 == 2 {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/cat_2.png".to_string(),
                    from: Vec2 { x: r, y: floor },
                    to: Vec2 { x: r, y: roof },
                });
            } else {
                commands.spawn(JumpPeopleImage {
                    img_name: "images/sedan_chair_2.png".to_string(),
                    from: Vec2 { x: che, y: floor },
                    to: Vec2 { x: che, y: roof },
                });
            }
        }
    }
}

// fn people_jump_system(mut evt: EventReader<AttackEvent>, mut commands: Commands) {
//     for evt in evt.read() {
//         //junp player
//     }
// }

// fn despawn_jump_image(mut evt: EventReader<TweenCompleted>, mut commands: Commands) {
//     for e in evt.read() {
//         commands.entity(e.entity).despawn();
//     }
// }

impl Plugin for ArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (create_people_system, gen_people));
    }
}

// impl Plugin for GameLevelUiPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(OnEnter(AppState::InGame), add_game_level_ui)
//             .add_systems(OnExit(AppState::InGame), remove_game_level_ui);
//     }
