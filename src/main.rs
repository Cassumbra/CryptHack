use bevy::{prelude::*, input::mouse::MouseMotion, render::render_resource::{AddressMode, FilterMode, Extent3d}};
use iyes_loopless::prelude::*;
use heron::prelude::*;
use leafwing_input_manager::{Actionlike, plugin::InputManagerPlugin, InputManagerBundle, prelude::{InputMap, ActionState}};


const SPEED: f32 = 12.;
const ACCELERATION: f32 = 2.;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity::from(Vec3::new(0., -9.81, 0.)))

        .add_state(GameState::Setup)

        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(InputManagerPlugin::<Action, GameState>::run_in_state(GameState::Playing))
        
        .add_stage_after(
            CoreStage::PreUpdate,
            "TransitionStage",
            StateTransitionStage::new(GameState::Setup)
        )

        .add_startup_system(setup.system())

        .add_system(load_textures.run_in_state(GameState::Setup))

        .add_system(process_actions.run_in_state(GameState::Playing))
        .add_system(cursor_grab_system.run_in_state(GameState::Playing))

        .run();
}


// Data
#[derive(Hash, PartialEq, Eq, Clone, Actionlike)]
pub enum Action {
    WalkForward,
    WalkBackward,
    StrafeLeft,
    StrafeRight,
    Jump,
    Crouch,
    //LookUp,
    //LookDown,
    //LookLeft,
    //LookRight,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Setup,
    Playing,
}

// Components
#[derive(Component)]
pub struct Player;


// Systems
fn process_actions(
    mut windows: ResMut<Windows>,
    time: Res<Time>,

    mut motion_evr: EventReader<MouseMotion>,

    mut camera_query: Query<(&mut Transform), (With<Camera>, With<Parent>)>,
    mut query: Query<(&Children, &ActionState<Action>, &mut Velocity, &mut Transform), Without<Camera>>
) {
    let sensitivity_mult = 0.005;
    let window = windows.get_primary_mut().unwrap();

    for (cameras, action_state, mut velocity, mut transform) in query.iter_mut() {
        if window.cursor_locked() {
            for ev in motion_evr.iter() {
                transform.rotate(Quat::from_rotation_y(-ev.delta.x * sensitivity_mult));
                for camera in cameras.iter() {
                    if let Ok(mut camera_transform) = camera_query.get_mut(*camera) {
                        camera_transform.rotate(Quat::from_rotation_x(-ev.delta.y * sensitivity_mult));
                        
                        //camera_transform.rotation.x - 
                        let lock_val = camera_transform.rotation.x.clamp(-0.523599, 0.523599) - camera_transform.rotation.x;
                        camera_transform.rotate(Quat::from_rotation_x(lock_val));
                    }
                }
            }
        }

        
        if action_state.just_pressed(&Action::Jump) {
            velocity.linear += Vec3::new(0., 5., 0.); 
        }

        let mut direction = Vec3::default();
        if action_state.pressed(&Action::WalkForward) {
            direction += -transform.local_z();
        }
        else if action_state.pressed(&Action::WalkBackward) {
            direction += transform.local_z();
        }

        if action_state.pressed(&Action::StrafeLeft) {
            direction += -transform.local_x();
        }
        else if action_state.pressed(&Action::StrafeRight) {
            direction += transform.local_x();
        }
        
        if direction != Vec3::default() {
            let mut velocity_add = ((direction.normalize_or_zero()*SPEED).lerp(velocity.linear, 0.0) - velocity.linear) * time.delta_seconds();
            velocity_add.y = 0.;
            velocity.linear += velocity_add;
        } else {
            let mut velocity_add = (velocity.linear.lerp(Vec3::default(), 1.0) - velocity.linear) * time.delta_seconds();
            velocity_add.y = 0.;
            velocity.linear += velocity_add;
        }
    }
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

fn setup (
    mut commands: Commands,

    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        /* 
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5,
                depth: 2.0,
                ..Default::default()
            })),
            material: materials.add(Color::ORANGE_RED.into()),
            ..Default::default()
        })
        */
        .insert(Transform {
            translation: Vec3::new(0., 15., 0.),
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
            radius: 0.5,
            half_segment: 1.0,
        })
        // Camera
        .with_children(|c| {
            c.spawn_bundle(PerspectiveCameraBundle::new_3d());
        });


        let room = assets.load("room.glb#Scene0");

        commands
            .spawn_scene(room);
        





    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(-4.0, 9.0, -4.0),
        ..Default::default()
    });
}

// TODO: how this could/should work:
// -Loop and make sure that all required textures are loaded
// -Proceed to setup stage after textures are loaded, spawn objects
fn load_textures (
    mut commands: Commands,

    mut game_state: ResMut<State<GameState>>,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let grass_handle: Handle<Image> = asset_server.load("textures/grass.png");


    if let Some(tex) = textures.get(grass_handle) {

        let mut grass_texture = tex.clone();


        //grass_texture.texture_descriptor.size = Extent3d { width: 64, height: 64, depth_or_array_layers: 2};

        grass_texture.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        grass_texture.sampler_descriptor.address_mode_v = AddressMode::Repeat;
        grass_texture.sampler_descriptor.address_mode_w = AddressMode::Repeat;
        grass_texture.sampler_descriptor.mag_filter = FilterMode::Nearest;
        grass_texture.sampler_descriptor.min_filter = FilterMode::Nearest;

        println!("{:?}", grass_texture.texture_descriptor.size);

        let handle = textures.add(grass_texture);
        
        // Floor
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(40., 1., 40.))),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(handle),
                    perceptual_roughness: 1.0,
                    metallic: 0.,
                    reflectance: 0.,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert_bundle((Transform::identity(), GlobalTransform::identity()))
            .insert(RigidBody::Static)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(20., 0.5, 20.),
                border_radius: None,
            });

        commands.insert_resource(NextState(GameState::Playing));
        game_state.set(GameState::Playing);
    }
}