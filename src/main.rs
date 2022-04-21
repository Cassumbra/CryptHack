use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};
use bevy_polyline::PolylinePlugin;
//use iyes_loopless::prelude::*;
use heron::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;

pub mod actions;
use actions::*;

pub mod player;
use player::*;

pub mod setup;
use setup::*;

pub mod map;
use map::*;

pub mod assets;
use assets::*;

fn main() {
    let mut app = App::new();
    AssetLoader::new(GameState::Loading)
        .continue_to_state(GameState::Setup)
        .with_collection::<TextureAssets>()
        .build(&mut app);

    app
        .add_state(GameState::Loading)
    
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity::from(Vec3::new(0., -9.81, 0.)))        

        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(InputManagerPlugin::<Action, GameState>::run_in_state(GameState::Playing))
        .add_plugin(PolylinePlugin)

        .add_plugin(MapPlugin)
        .add_plugin(AssetPlugin)

        //.add_stage_after(
        //    CoreStage::PreUpdate,
        //    "TransitionStage",
        //    StateTransitionStage::new(GameState::Setup)
        //)

        .add_system_set(
            SystemSet::on_exit(GameState::Loading)
                .with_system(create_assets.label("create_assets"))
        )

        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            SystemSet::on_enter(GameState::Setup)
                .with_system(map_branching_start.label("start_map"))
                .with_system(setup.after("start_map"))
        )

        .add_system_set(
            SystemSet::on_update(GameState::Setup)
                .with_system(map_branching_generation)
        )

        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(process_actions)
                .with_system(cursor_grab_system)
        )

        .add_system(check_scene_objects)
        .add_system(spawn_surface)
        .add_system(spawn_surfaces)

        .run();
}


// Data
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    Setup,
    Playing,
}