use bevy::{prelude::*};
use heron::{prelude::*, PendingConvexCollision};
use iyes_loopless::state::NextState;
use leafwing_input_manager::prelude::*;

use crate::{actions::Action, player::Player, map::{geometric::Rect3Room, MapScale}, GameState};

//use super::{GameState, TextureAssets};

const PLAYER_HEIGHT: f32 = 0.4;

// Systems
pub fn spawn_actors (
    mut commands: Commands,

    map_scale: Res<MapScale>,

    room_query: Query<&Rect3Room>,
) {
    println!("Spawning actors");

    let mut spawn_pos = Vec3::new(0.0, 1.0, 0.0);
    if let Some(room) = room_query.iter().next() {
        println!("Setting player position to room center");
        spawn_pos = room.rect.center();
    }
    
    spawn_pos *= **map_scale;

    // Player
    commands
        .spawn_bundle(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([(Action::Jump, KeyCode::Space),
                                      (Action::Crouch, KeyCode::LControl),
                                      (Action::StrafeRight, KeyCode::D),
                                      (Action::StrafeLeft, KeyCode::A),
                                      (Action::WalkForward, KeyCode::W),
                                      (Action::WalkBackward, KeyCode::S),
                                     ])
        })
        .insert(Player)
        .insert(Transform {
            translation: spawn_pos,
            ..default()
        })
        .insert(GlobalTransform::identity())
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(RotationConstraints {
            allow_x: false,
            allow_y: false,
            allow_z: false,
        })
        .insert(CollisionShape::Capsule {
            radius: 0.2,
            half_segment: PLAYER_HEIGHT / 2.0,
        })
        // Camera
        .with_children(|c| {
            c.spawn_bundle(PerspectiveCameraBundle::new_3d())
                .insert( Transform {
                    translation: Vec3::new(0.0, PLAYER_HEIGHT - (PLAYER_HEIGHT / 4.0), 0.0),
                    ..default()
                });
        })
        // light
        .with_children(|c| {
            c.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 200.0,
                    //range: 80.0,
                    //radius: 80.0,
                    ..default()
                },
                ..default()
            });
        });

    commands.insert_resource(NextState(GameState::Playing));
}

pub fn check_scene_objects (
    mut commands: Commands,
    entities: Query<(Entity, &Name), Added<Name>>
) {
    for (entity, name) in entities.iter() {
        if name.as_str().contains("Collidable") {
            commands.entity(entity).insert(PendingConvexCollision::default());
        }
    }
}