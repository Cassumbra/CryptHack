

#![windows_subsystem = "windows"]

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use heron::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;

pub mod actions;
use actions::*;

pub mod player;
use player::*;

pub mod setup;
use setup::*;

#[path = "map/map.rs"]
pub mod map;
use map::*;

pub mod assets;
use assets::*;

fn main() {
    let mut app = App::new();

    app
        .insert_resource(WindowDescriptor{
            title: "CryptHack".to_string(),
            resizable: true,
            ..Default::default()}
        )

        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity::from(Vec3::new(0., -9.81, 0.)))        

        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(InputManagerPlugin::<Action>::default())

        .add_plugin(MapPlugin)
        .add_plugin(AssetPlugin)

        .add_stage_after(CoreStage::Update, "Update Geometry", SystemStage::parallel())

        .add_loopless_state(GameState::Loading)

        .add_enter_system(GameState::Loading, create_assets)

        //.add_enter_system(GameState::StartMapGen, map_branching_start)
        .add_system(map_branching_start.run_in_state(GameState::StartMapGen))

        .add_system(map_branching_generation.run_in_state(GameState::MapGen))

        .add_enter_system(GameState::SpawnActors, spawn_actors)


        // TODO: Change this once asset_loader supports loopless.
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .with_system(process_actions)
                .with_system(meta_input)
                .into()
        )

        .add_system(check_scene_objects)

        .add_system_set_to_stage(
            "Update Geometry",
            SystemSet::new()
                .with_system(spawn_rooms)
                .with_system(spawn_exits.after(spawn_rooms))
                .with_system(spawn_entrances.after(spawn_rooms))
        )


        //.add_system(spawn_surface)
        //.add_system(spawn_surfaces)

        .run();
}


// Data
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    StartMapGen, MapGen, SpawnActors,
    Playing,
}