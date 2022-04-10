use bevy::{prelude::*, input::mouse::MouseMotion, render::render_resource::{AddressMode, FilterMode, Extent3d}, gltf::{Gltf, self}};
use bevy_asset_loader::{AssetLoader, AssetCollection};
use iyes_loopless::prelude::*;
use heron::prelude::*;
use leafwing_input_manager::{Actionlike, plugin::InputManagerPlugin, InputManagerBundle, prelude::{InputMap, ActionState}};


const SPEED: f32 = 12.;
const ACCELERATION: f32 = 2.;

fn main() {
    let mut app = App::new();
    AssetLoader::new(GameState::Loading)
        .continue_to_state(GameState::Setup)
        .with_collection::<TextureAssets>()
        .with_collection::<SceneAssets>()
        .build(&mut app);

    app
        .add_state(GameState::Loading)
    
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity::from(Vec3::new(0., -9.81, 0.)))        

        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(InputManagerPlugin::<Action, GameState>::run_in_state(GameState::Playing))
        
        //.add_stage_after(
        //    CoreStage::PreUpdate,
        //    "TransitionStage",
        //    StateTransitionStage::new(GameState::Setup)
        //)


         // TODO: Change this once asset_loader supports loopless.
         .add_system_set(
             SystemSet::on_update(GameState::Loading)
                 .with_system(test)
         )

        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            SystemSet::on_enter(GameState::Setup)
                .with_system(setup)
        )

        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(process_actions)
                .with_system(cursor_grab_system)
        )

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
    Loading,
    Setup,
    Playing,
}

#[derive(AssetCollection)]
struct SceneAssets {
  #[asset(path = "room.glb#Scene0")]
  room: Handle<Scene>
}

#[derive(AssetCollection)]
struct TextureAssets {
  #[asset(path = "textures/grass.png")]
  grass: Handle<Image>,
}

// Components
#[derive(Component)]
pub struct Player;


// Systems
fn test (
    
) {
    println!("Loading!");
}

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

    mut game_state: ResMut<State<GameState>>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_handles: Res<SceneAssets>,
    texture_handles: Res<TextureAssets>,
    mut gltfs: ResMut<Assets<Gltf>>,
    mut textures: ResMut<Assets<Image>>,
) {     
    // Floor
    /*
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(40., 1., 40.))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(texture_handles.grass.clone()),
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
     */
    
    


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


        //let mut room = gltfs.get(gltf_handles.room.clone()).unwrap();

        //for obj in room.meshes.iter() {

        //}

        commands
            .spawn_scene(scene_handles.room.clone());
        
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(-4.0, 9.0, -4.0),
        ..Default::default()
    });

    game_state.set(GameState::Playing);
    //commands.insert_resource(NextState(GameState::Playing));
}

fn check_scene_objects (
    mut commands: Commands,
    entities: Query<(Entity, &Name), Added<Name>>
) {

}