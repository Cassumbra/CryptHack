use std::ops::Deref;

use bevy::prelude::*;

pub mod geometric;
pub use geometric::*;

pub mod grid;
pub use grid::*;
use rand::{Rng, prelude::SliceRandom};

use super::GameState;
use super::assets::TileAssets;

// Rooms are equivalent to Nodes. Branches are equivalent to Edges.

const MIN_ROOMS: usize = 1;
const MAX_ROOMS: i32 = 1;

const MIN_BRANCHES_FROM_ROOM: i32 = 1;
const MAX_BRANCHES_FROM_ROOM: i32 = 3;

// Room/Node generation
const MIN_SIZE: i32 = 6;
const MAX_SIZE: i32 = 10;
const MIN_HEIGHT: i32 = 1;
const MAX_HEIGHT: i32 = 3;

// Branch/Edge generation.
const MIN_TURNS: i32 = 0;
const MAX_TURNS: i32 = 4;
const MIN_DIST: i32 = 5;
const MAX_DIST: i32 = 10;

// Plugin
#[derive(Default)]
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RoomSpawnAttempts>()
            .init_resource::<GridMap>();
    }
}

// Systems
pub fn map_branching_start (
    map: Res<GridMap>,
    tiles: Res<TileAssets>,

    mut room_spawn_attempts: ResMut<RoomSpawnAttempts>,
    mut game_state: ResMut<State<GameState>>,

    mut commands: Commands,
) {
    
    let mut rng = rand::thread_rng();
    
    let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
    let h = rng.gen_range(MIN_HEIGHT..MAX_HEIGHT);
    let l = rng.gen_range(MIN_SIZE..MAX_SIZE);
    let x = rng.gen_range(0..(map.width() - w));
    let y = rng.gen_range(0..(map.height() - h));
    let z = rng.gen_range(0..(map.length() - l));

    let room = Rect3Room {
        ceiling: tiles.grass.clone(),
        walls: tiles.grass.clone(),
        floor: tiles.grass.clone(),
        rect: Rect3::new(IVec3::new(x, y, z), w, h, l),

        ..default()
    };
    let entrances = Entrances(Vec::new());
    let exits = Exits(Vec::new());
    
    commands
        .spawn()
        .insert(room)
        .insert(entrances)
        .insert(exits);

    **room_spawn_attempts = 1;

    game_state.set(GameState::MapGen);
}


pub fn map_branching_generation (
    mut map: ResMut<GridMap>,
    mut game_state: ResMut<State<GameState>>,
    mut room_spawn_attempts: ResMut<RoomSpawnAttempts>,

    room_query: Query<(Entity, &Rect3Room, &Entrances, &Exits)>,

    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    let rooms = room_query.iter().collect::<Vec<(Entity, &Rect3Room, &Entrances, &Exits)>>();

    if **room_spawn_attempts >= MAX_ROOMS {
        if rooms.len() < MIN_ROOMS {
            // TODO: restart generation from here
            // Maybe clear what is currently generated and then move back to map_branching_start?
            for position in &*map {
                clear_tile(&mut commands, &mut map, position);
            }
            for (entity, _, _, _) in room_query.iter() {
                commands.entity(entity).despawn();
            }
            // TODO: Delete entrances
            // TODO: Delete exits

            game_state.set(GameState::StartMapGen);
        }
        else {
            // Finish generation

            game_state.set(GameState::SpawnActors);
        }
    }



    let mut room = rooms.choose_weighted(&mut rng, |(_ent, _room, entrances, exits)| 1/(entrances.len() + exits.len() + 1));


    // TODO: Actual stuff here


}

pub fn spawn_rooms (
    mut map: ResMut<GridMap>,

    room_query: Query<(Entity, &Rect3Room), (Added<Rect3Room>)>,

    mut commands: Commands,
) {
    for (entity, room) in room_query.iter() {
        let min = room.rect.min();
        let max = room.rect.max();

        for position in room {
            println!("{:?}", position);
            clear_tile(&mut commands, &mut map, position);
    
            
            if position.y == max.y {
                spawn_tile(&mut commands, &mut map, room.ceiling.clone(), TileType::Ceiling, position);
            }
            if position.z == max.z {
                spawn_tile(&mut commands, &mut map, room.walls.clone(), TileType::North, position);
            }
            if position.x == max.x {
                spawn_tile(&mut commands, &mut map, room.walls.clone(), TileType::East, position);
            }

            if position.y == min.y {
                spawn_tile(&mut commands, &mut map, room.floor.clone(), TileType::Floor, position);
            }
            if position.z == min.z {
                spawn_tile(&mut commands, &mut map, room.walls.clone(), TileType::South, position);
            }
            if position.x == min.x {
                spawn_tile(&mut commands, &mut map, room.walls.clone(), TileType::West, position);
            }
        }
    }
}

pub fn spawn_exits (

) {

}

pub fn spawn_entrances (

) {

}

// Data
pub struct WithinBoxIterator {
    position: IVec3,
    min: IVec3,
    max: IVec3,
}
impl Iterator for WithinBoxIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        self.position.x += 1;
        if self.position.x > self.max.x {
            self.position.x = self.min.x;
            self.position.y += 1;
        }
        if self.position.y > self.max.y {
            self.position.y = self.min.y;
            self.position.z += 1;
        }
        if self.position.z > self.max.z {
            return None
        }
        else {
            return Some(self.position)
        }
    }
}
impl WithinBoxIterator {
    pub fn new(min: IVec3, max: IVec3) -> WithinBoxIterator {
        WithinBoxIterator{position: min + IVec3::new(-1, 0, 0), min, max}
    }
}

// Resources
#[derive(Default, Deref, DerefMut, Clone)]
pub struct RoomSpawnAttempts(i32);