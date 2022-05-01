use std::f32::consts::E;
use std::ops::Deref;

use bevy::prelude::*;

pub mod geometric;
pub use geometric::*;

pub mod grid;
pub use grid::*;
use iyes_loopless::state::NextState;
use rand::{Rng, prelude::SliceRandom};

use super::GameState;
use super::assets::TileAssets;

// Rooms are equivalent to Nodes. Branches are equivalent to Edges.

const MIN_ROOMS: usize = 3;
const MAX_ROOMS: i32 = 30;

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
const MIN_DIST: i32 = 3;
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

    mut commands: Commands,
) {
    println!("starting map gen");

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

    commands.insert_resource(NextState(GameState::MapGen));
}


pub fn map_branching_generation (
    mut map: ResMut<GridMap>,
    mut room_spawn_attempts: ResMut<RoomSpawnAttempts>,

    mut room_query: ParamSet<(
        Query<(Entity, &Entrances, &Exits), With<Rect3Room>>,
        Query<(&Rect3Room, &Entrances, &mut Exits)>,
    )>,
        
    //mut room_query: Query<(Entity, &Rect3Room, &mut Entrances, &Exits)>,
    entrance_query: Query<(Entity, &HoleEntrance)>,
    exit_query: Query<(Entity, &PathExit)>,

    mut commands: Commands,
) {
    println!("branching gen");

    let mut rng = rand::thread_rng();

    let room_query_0 = room_query.p0();
    let rooms = room_query_0.iter().collect::<Vec<(Entity, &Entrances, &Exits)>>();

    // TODO: These should make us leave this system immediately.
    if **room_spawn_attempts >= MAX_ROOMS {
        println!("{:?}", rooms.len());
        if rooms.len() < MIN_ROOMS {
            // TODO: restart generation from here
            // Maybe clear what is currently generated and then move back to map_branching_start?
            println!("Restarting generation.");

            for position in &*map {
                clear_position(&mut commands, &mut map, position);
            }
            for (entity, _, _) in &rooms {
                commands.entity(*entity).despawn();
            }
            for (entity, _) in entrance_query.iter() {
                commands.entity(entity).despawn();
            }
            for (entity, _) in exit_query.iter() {
                commands.entity(entity).despawn();
            }

            commands.insert_resource(NextState(GameState::StartMapGen));
            return;
        }
        else {
            // Finish generation
            println!("Generation finished");

            commands.insert_resource(NextState(GameState::SpawnActors));
        }
    }

    /*
    println!("WEIGHTS");
    for (ent, entrances, exits) in &rooms {
        println!("{:?} entrances {:?}", ent, entrances.len());
        println!("{:?} exits {:?}", ent, exits.len());
        println!("{:?} weight is: {:?}", ent, 1.0 / (entrances.len() + exits.len() + 1) as f32)
    }
     */

    let room_entity = rooms.choose_weighted(&mut rng, |(_ent, entrances, exits)| 1.0 / (entrances.len() + exits.len() + 1) as f32).unwrap().0;

    if let Ok((room, entrances, mut exits)) = room_query.p1().get_mut(room_entity) {
        //let mut entrances = Vec::new();
        //let mut exits = Vec::new();
        let mut exclude = Vec::new();

        // This feels clunky and messy and a bit out of place. Perhaps it should be handled differently?? Not sure how though. It might be fine tbh.
        for entrance_entity in entrances.iter() {
            if let Ok((entrance_entity, entrance)) = entrance_query.get(*entrance_entity) {
                exclude.push(entrance.position);
            }
            else {
                // This feels messy and silly and is going to cause problems if we add different entrance types.
                // Guess we'll deal with that later.
                // Maybe we can just add an else if above this.
                panic!("Room contains non-entrance entrance!");
            }
        }

        for exit_entity in exits.iter() {
            if let Ok((exit_entity, exit)) = exit_query.get(*exit_entity) {
                exclude.push(exit.path[0].position);
            }
            else {
                panic!("Room contains non-exit exit!");
            }
        }

        let mut exit = PathExit {
            ceiling: room.ceiling.clone(),
            walls: room.walls.clone(),
            floor: room.floor.clone(),
            ..default()
        };

        let mut path_positions = Vec::new();

        let mut can_spawn_room = true;

        if let Some((exit_point, exit_orientation)) = random_surface_wall_point(exclude, room.rect, &*map) {
            // TODO: Get to work on path generation.
            let mut vector = TileOffsets::default()[exit_orientation].translation * 2.0;
            let mut current_point = exit_point;
            let mut current_orientation = exit_orientation;

            exit.path.push(IVec3Tile::new(current_point, current_orientation));

            let turns = rng.gen_range(MIN_TURNS..=MAX_TURNS - 1) + 1;
            'path: for t in 0..=turns {
                let turn_left = rng.gen_bool(0.5);
                let distance = rng.gen_range(MIN_DIST..=MAX_DIST);
                for _ in 0..distance {
                    current_point += IVec3::new(vector.x as i32, vector.y as i32, vector.z as i32);


                    // Check if path intersects itself
                    path_positions = exit.path.iter().map(|path| path.position).collect::<Vec<IVec3>>();

                    if path_positions.contains(&current_point) {
                        can_spawn_room = false;
                        exit.path.push(IVec3Tile::new(current_point, current_orientation));
                        break 'path;
                    }

                    // Check if path is out of bounds
                    if map.position_oob(current_point) {
                        can_spawn_room = false;
                        break 'path;
                    }
                    // Push the current point and location if we aren't out of bounds
                    else {
                        exit.path.push(IVec3Tile::new(current_point, current_orientation));
                    }

                    // Check if path intersects with anything else
                    if map.position_collides(current_point){
                        can_spawn_room = false;
                        break 'path;
                    }
                }

                // Take a turn if applicable
                if t != turns {
                    current_orientation = current_orientation.rotate90(turn_left);
                    vector = TileOffsets::default()[current_orientation].translation * 2.0;
                }
            }

            if can_spawn_room {
                let mut pos1 = exit.path.last().unwrap().position;
                let orientation = exit.path.last().unwrap().orientation;

                let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
                let h = rng.gen_range(MIN_HEIGHT..MAX_HEIGHT);
                let l = rng.gen_range(MIN_SIZE..MAX_SIZE);

                if orientation == TileType::East || orientation == TileType::West {
                    pos1.z -= l/2;
                }
                else if orientation == TileType::North || orientation == TileType::South {
                    pos1.x -= w/2;
                }

                let rect = Rect3::new(pos1, w, h, l);

                let mut is_ok = true;

                // Don't include last position in path in checking.
                path_positions.pop();

                for position in rect {
                    if map.position_oob(position) || map.position_collides(position) || path_positions.contains(&position) {
                        is_ok = false;
                        break;
                    }
                }

                if is_ok {
                    let room = Rect3Room {
                        ceiling: exit.ceiling.clone(),
                        walls: exit.walls.clone(),
                        floor: exit.floor.clone(),
                        rect,
                
                        ..default()
                    };
                    
                    let entrance = IVec3Tile::new(exit.path.last().unwrap().position, orientation);

                    let entrance_entity = commands.spawn()
                                                  .insert(HoleEntrance(entrance))
                                                  .id();

                    commands
                        .spawn()
                        .insert(room)
                        .insert(Entrances(vec![entrance_entity]))
                        .insert(Exits(Vec::new()));

                    let exit_entity = commands.spawn().insert(exit).id();
                    exits.push(exit_entity);
                }
            }
            else {
                println!("no room...: {:?}", exit.path.clone());

                let exit_entity = commands.spawn().insert(exit).id();
                exits.push(exit_entity);
            }




        }
    }

    **room_spawn_attempts += 1;
}


// TODO: Entities should be children of their room.
pub fn spawn_rooms (
    mut map: ResMut<GridMap>,

    room_query: Query<(Entity, &Rect3Room), (Added<Rect3Room>)>,

    mut commands: Commands,
) {
    for (entity, room) in room_query.iter() {
        println!("spawning room");
        let min = room.rect.min();
        let max = room.rect.max();

        for position in room {
            clear_position(&mut commands, &mut map, position);
    
            
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

// TODO: Entities should be children of their path.
pub fn spawn_exits (
    mut map: ResMut<GridMap>,

    path_query: Query<(Entity, &PathExit), (Added<PathExit>)>,

    mut commands: Commands,
) {
    for (entity, exit) in path_query.iter() {
        println!("spawning exit");

        for (i, p) in exit.path.iter().enumerate() {
            // Start
            if i == 0 {
                clear_tile(&mut commands, &mut map, p.orientation, p.position);
            }
            // End
            else if exit.path.len() - 1 == i {
                clear_tile(&mut commands, &mut map, p.orientation.rotate90(true).rotate90(true), p.position);
            }
            // Anywhere inbetween
            else {
                clear_position(&mut commands, &mut map, p.position);

                if exit.path[i+1].orientation == p.orientation {
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation.rotate90(true), p.position);
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation.rotate90(false), p.position);
                }
                else if exit.path[i+1].orientation == p.orientation.rotate90(true) {
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation, p.position);
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation.rotate90(false), p.position);
                }
                else if exit.path[i+1].orientation == p.orientation.rotate90(false) {
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation, p.position);
                    spawn_tile(&mut commands, &mut map, exit.walls.clone(), p.orientation.rotate90(true), p.position);
                }
                else {
                    panic!("Malformed path!");
                }

                spawn_tile(&mut commands, &mut map, exit.walls.clone(), TileType::Ceiling, p.position);
                spawn_tile(&mut commands, &mut map, exit.walls.clone(), TileType::Floor, p.position);


            }
        }
    }
}

pub fn spawn_entrances (

) {

}

// Helper Functions

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