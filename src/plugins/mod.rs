pub mod art;
pub mod character_selection;
pub mod game_level;
pub mod input;
pub mod ringcon;
pub mod score;
pub mod sound_player;
pub mod start_menu;

use std::time::Duration;

use crate::config::ImageKey;
use bevy::{prelude::*, sprite::Anchor};
use bevy_tweening::{
    lens::{SpriteColorLens, TransformPositionLens},
    Animator, EaseFunction, Tween, TweenCompleted,
};

#[derive(Debug, Component)]
pub struct JumpImage {
    pub(crate) key: ImageKey,
    pub(crate) from: Vec2,
    pub(crate) to: Vec2,
}

/// play one-shot SFV
pub struct PlaySfxEvent;

pub struct JumpImagePlugin;

#[derive(Debug, Event)]
pub struct RemoveEntity(Entity);

fn spawn_jump_image(
    mut commands: Commands,
    query: Query<(Entity, &JumpImage)>,
    asset_server: Res<AssetServer>,
) {
    for (ent, img) in &query {
        let from = Vec3::new(img.from.x, img.from.y, 1.);
        let to = Vec3::new(img.to.x, img.to.y, 1.);
        let img: Handle<Image> = asset_server.load::<Image>(format!("images/{}", img.key));

        let duration = Duration::from_secs_f32(0.8);
        let pos_tween = Tween::new(
            EaseFunction::CircularInOut,
            duration,
            TransformPositionLens {
                start: from,
                end: to,
            },
        );
        let color_tween = Tween::new(
            EaseFunction::CircularInOut,
            duration,
            SpriteColorLens {
                start: Color::default(),
                end: Color::rgba(1., 1., 1., 0.5),
            },
        )
        .with_completed_event(0);

        commands.spawn((
            // Spawn a Sprite entity to animate the position of.
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::Center,
                    ..Default::default()
                },
                transform: Transform {
                    translation: from,
                    ..default()
                },
                texture: img,
                ..default()
            },
            // Add an Animator component to control and execute the animation.
            Animator::new(pos_tween),
            Animator::new(color_tween),
        ));
        // remove ent
        commands.entity(ent).despawn();
    }
}

fn despawn_jump_image(mut evt: EventReader<TweenCompleted>, mut commands: Commands) {
    for e in evt.read() {
        commands.entity(e.entity).despawn();
    }
}

impl Plugin for JumpImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_jump_image, despawn_jump_image));
    }
}
