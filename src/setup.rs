use bevy::{prelude::*, gltf::Gltf};
use heron::{prelude::*, PendingConvexCollision};
use leafwing_input_manager::prelude::*;

use crate::{actions::Action, player::Player};

use super::{GameState, SceneAssets, TextureAssets};

const PLAYER_HEIGHT: f32 = 0.4;

// Systems
pub fn setup (
    mut commands: Commands,

    mut game_state: ResMut<State<GameState>>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_handles: Res<SceneAssets>,
    texture_handles: Res<TextureAssets>,
    mut gltfs: ResMut<Assets<Gltf>>,
    mut textures: ResMut<Assets<Image>>,
) {     
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
            translation: Vec3::new(0., 1., 0.),
            ..Default::default()
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
                    ..Default::default()
                });
        })
        // light
        .with_children(|c| {
            c.spawn_bundle(PointLightBundle {
                ..Default::default()
            });
        });


    //let mut room = gltfs.get(gltf_handles.room.clone()).unwrap();

    //for obj in room.meshes.iter() {

    //}

    //commands
    //    .spawn_scene(scene_handles.room.clone());
        






    game_state.set(GameState::Playing);
    //commands.insert_resource(NextState(GameState::Playing));
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