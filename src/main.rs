use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};
//use iyes_loopless::prelude::*;
use heron::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;

pub mod actions;
use actions::*;

pub mod player;
use player::*;

pub mod setup;
use setup::*;


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
            SystemSet::on_enter(GameState::Setup)
                .with_system(setup)
        )

        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(process_actions)
                .with_system(cursor_grab_system)
        )

        .add_system(check_scene_objects)

        .run();
}


// Data
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    Setup,
    Playing,
}

#[derive(AssetCollection)]
pub struct SceneAssets {
  #[asset(path = "room.glb#Scene0")]
  room: Handle<Scene>
}

#[derive(AssetCollection)]
pub struct TextureAssets {
  #[asset(path = "textures/grass.png")]
  grass: Handle<Image>,
}