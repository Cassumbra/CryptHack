use bevy::prelude::*;
use heron::prelude::*;

use enum_map::{EnumMap, Enum, enum_map};
use rand::{Rng, prelude::{SliceRandom, IteratorRandom}};
use ndarray::{Array3, s};
use delaunay3d::*;

use crate::assets::{TileAssets, MeshAssets};

use super::{TextureAssets, MaterialAssets, GameState};


//const MAX_ROOMS: i32 = 30;
const MIN_SIZE: i32 = 6;
const MAX_SIZE: i32 = 10;
const MIN_HEIGHT: i32 = 1;
const MAX_HEIGHT: i32 = 3;

const MIN_TURNS: i32 = 0;
const MAX_TURNS: i32 = 4;
const MIN_DIST: i32 = 5;
const MAX_DIST: i32 = 10;

//Plugin
#[derive(Default)]
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TileOffsets>()
            .init_resource::<Map>()
            .add_event::<MapChangeEvent>()
            .add_event::<SpawnSurfaceEvent>()
            .add_event::<SpawnSurfacesEvent>();
    }
}

// Systems
pub fn map_branching_start (
    mut map: ResMut<Map>,
    tiles: Res<TileAssets>,
) {
    let mut rng = rand::thread_rng();

    let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
    let h = rng.gen_range(MIN_HEIGHT..MAX_HEIGHT);
    let l = rng.gen_range(MIN_SIZE..MAX_SIZE);
    let x = rng.gen_range(0..(map.width - w));
    let y = rng.gen_range(0..(map.height - h));
    let z = rng.gen_range(0..(map.length - l));

    let room = Rect3::new(IVec3::new(x, y, z), w, h, l);

    

    map.rooms.push(Room::simple_rect(
        room,
        Vec::new(),
        Vec::new(),
        tiles.grass.clone(),
    ));

    map.update_tiles();

    let exits = map.rand_surface_wall_points(9, 9, &room);

    map.rooms[0].map_empty_doorways(exits, tiles.grass.clone());
}

/// Create geometric map representation over time
pub fn map_branching_generation (
    mut ev_spawn_surface: EventWriter<SpawnSurfaceEvent>,  

    mut map: ResMut<Map>,
    mut game_state: ResMut<State<GameState>>,

    material_handles: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    tiles: Res<TileAssets>,

    tile_offsets: Res<TileOffsets>,
) {
    let mut rng = rand::thread_rng();
    let mut generation_done = true;

    // Mutable vec for the borrow checker.
    let mut rooms = Vec::<(Rect3, Entrance)>::new();

    let room_num = map.rooms.len();
    for room in map.rooms.iter_mut().choose_multiple(&mut rng, room_num) {
        let exit_num = room.exits.len();
        for exit in room.exits.iter_mut().choose_multiple(&mut rng, exit_num) {
            match &mut exit.exit_type {
                ExitType::Doorway { location, orientation, path, need_room, ceiling, walls, floor } => {
                    let mut current_orientation = orientation.clone();

                    if path.is_empty() {
                        generation_done = false;

                        let mut vector = tile_offsets.0[current_orientation].translation * 2.0;
                        let mut current_location = location.clone();

                        let turns = rng.gen_range(MIN_TURNS..=MAX_TURNS - 1) + 1;
                        for t in 0..=turns {
                            let turn_left = rng.gen_bool(0.5);
                            let distance = rng.gen_range(MIN_DIST..=MAX_DIST);
                            for _ in 0..distance {
                                current_location += IVec3::new(vector.x as i32, vector.y as i32, vector.z as i32);
                                path.push(current_location);
                            }

                            if t != turns {
                                current_orientation = current_orientation.rotate90(turn_left);
                                vector = tile_offsets.0[current_orientation].translation * 2.0;
                            }
                        }

                        exit.spawned = false;
                    }

                    if *need_room {
                        generation_done = false;
                     
                        let vector = path[path.len() - 1] - path[path.len() - 2];
                        let next = path[path.len() - 1] + vector;
                        let mut pos1 = next;

                        let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
                        let h = rng.gen_range(MIN_HEIGHT..MAX_HEIGHT);
                        let l = rng.gen_range(MIN_SIZE..MAX_SIZE);

                        if vector.x != 0 {
                            pos1.z -= l/2;
                        }
                        else if vector.z != 0 {
                            pos1.x -= w/2;
                        }

                        let rect = Rect3::new(pos1, w, h, l);
                        println!("normal: {:?}", current_orientation);
                        println!("rotated 180: {:?}", current_orientation.rotate90(true).rotate90(true));
                        println!("normal (again): {:?}", current_orientation);
                        let entrance = Entrance{location: next, orientation: current_orientation.rotate90(true).rotate90(true)};

                        // The borrow checker must be appeased.
                        rooms.push((rect, entrance));


                    }
                }

                _ => { }
            }

            
        }
    }

    // We must appease the borrow checker.
    for (room, entrance) in rooms {
        if rect_valid(IVec3::new(map.width, map.height, map.length), room) && !rect_intersects(&map, room) {
            map.rooms.push(
                Room::simple_rect(
                    room,
                    Vec::new(),
                    vec![entrance],
                    tiles.grass.clone(),
                )
            );
        }
        map.update_tiles();
    }

    map.update_tiles();

    if generation_done {
        for ((x, y, z), section) in map.tiles.indexed_iter() {
            for (tile_type, opt_tile) in section.iter() {
                if let Some(tile) = opt_tile {
                    // TODO:
                    // Perhaps in the future we can use spawn_surfaces and change that system to only generate one collision object (while still generating lots of entities for tiling purposes)
                    // This would create smoother surfaces with less jank (probably???)
                    // Although also in the future we should still fix that lots of entities thing...
                    ev_spawn_surface.send(SpawnSurfaceEvent { material: tile.material.clone(), mesh: meshes.plane.clone(), location: Vec3::new(x as f32, y as f32, z as f32), tile_type: tile_type });
    
                }
            }
        }

        game_state.set(GameState::Playing);
        //commands.insert_resource(NextState(GameState::Playing));
    }
}

pub fn spawn_surface (
    mut ev_spawn_surface: EventReader<SpawnSurfaceEvent>,
    mut commands: Commands,
    offsets: Res<TileOffsets>,
) {
    for ev in ev_spawn_surface.iter() {
        // Rename to offset?
        let transformation = offsets.0[ev.tile_type];
        let mut transform = Transform::default();

        transform.translation = ev.location + transformation.translation;

        if transformation.rotation.x != 0.0 {
            transform.rotate(Quat::from_rotation_x(transformation.rotation.x))
        }
        if transformation.rotation.y != 0.0 {
            transform.rotate(Quat::from_rotation_y(transformation.rotation.y))
        }
        if transformation.rotation.z != 0.0 {
            transform.rotate(Quat::from_rotation_z(transformation.rotation.z))
        }

        commands
            .spawn_bundle(PbrBundle {
                mesh: ev.mesh.clone(),
                material: ev.material.clone(),
                ..Default::default()    
            })  
            .insert(transform)
            .insert(GlobalTransform::default())
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(0.5, 0.0, 0.5),
                border_radius: None,
            })
            .insert(RigidBody::Static)
            .insert(CollisionLayers::default());
    }
}

pub fn spawn_surfaces (
    mut ev_spawn_surfaces: EventReader<SpawnSurfacesEvent>,
    mut commands: Commands,
    offsets: Res<TileOffsets>,
) {
    for ev in ev_spawn_surfaces.iter() {
        for x in ev.location1.x as i32..ev.location2.x as i32 {
            for y in ev.location1.y as i32..ev.location2.y as i32 {
                for z in ev.location1.z as i32..ev.location2.z as i32 {
                    // Rename to offset?
                    let transformation = offsets.0[ev.tile_type];
                    let mut transform = Transform::default();

                    transform.translation = Vec3::new(x as f32, y as f32, z as f32) + transformation.translation;

                    if transformation.rotation.x != 0.0 {
                        transform.rotate(Quat::from_rotation_x(transformation.rotation.x))
                    }
                    if transformation.rotation.y != 0.0 {
                        transform.rotate(Quat::from_rotation_y(transformation.rotation.y))
                    }
                    if transformation.rotation.z != 0.0 {
                        transform.rotate(Quat::from_rotation_z(transformation.rotation.z))
                    }

                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: ev.mesh.clone(),
                            material: ev.material.clone(),
                            ..Default::default()    
                        })  
                        .insert(transform)
                        .insert(GlobalTransform::default())
                        .insert(CollisionShape::Cuboid {
                            half_extends: Vec3::new(0.5, 0.0, 0.5),
                            border_radius: None,
                        })
                        .insert(RigidBody::Static)
                        .insert(CollisionLayers::default());
                }
            }
        }
    }
}

// Helper functions
// sorry nothing

// Events
pub struct SpawnSurfaceEvent {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    location: Vec3,
    tile_type: TileType,
}

pub struct SpawnSurfacesEvent {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    location1: Vec3,
    location2: Vec3,
    tile_type: TileType,
}

pub struct MapChangeEvent;


// Data
#[derive(Enum, Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Center,
    Ceiling,
    Floor,
    North,
    East,
    South,
    West,
}
impl TileType {
    pub fn rotate90(&mut self, left: bool) -> TileType {
        /*
        let cardinals = vec![TileType::North, TileType::East, TileType::South, TileType::West];
        let mut index = cardinals.iter().position(|&x| x == *self).unwrap();

        if left {
            if index == 0 {
                index = cardinals.len() - 1;
            } else {
                index -= 1;
            }
            
        } else {
            if index == cardinals.len() - 1 {
                index = 0;
            } else {
                index += 1;
            }
            
        }


        *self = cardinals[index];
         */

        if left {
            match self {
                TileType::North => TileType::West,
                TileType::West => TileType::South,
                TileType::South => TileType::East,
                TileType::East => TileType::North,

                _ => TileType::Center,
            }
        } else {
            match self {
                TileType::North => TileType::East,
                TileType::East => TileType::South,
                TileType::South => TileType::West,
                TileType::West => TileType::North,

                _ => TileType::Center,
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Transformation {
    translation: Vec3,
    rotation: Vec3,
}
impl Transformation {
    fn with_translation(translate: Vec3) -> Transformation {
        Transformation {
            translation: translate,
            rotation: Vec3::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TileOffsets (EnumMap<TileType, Transformation>);
impl Default for TileOffsets {
    fn default() -> TileOffsets {
        TileOffsets(
            enum_map! {
                TileType::Center => Transformation::default(),

                TileType::Ceiling => Transformation {translation: Vec3::new(0.0, 0.5, 0.0), rotation: Vec3::new(180.0_f32.to_radians(), 0.0, 0.0)},
                TileType::Floor => Transformation {translation: Vec3::new(0.0, -0.5, 0.0), rotation: Vec3::new(0.0, 0.0, 0.0)},

                TileType::North => Transformation {translation: Vec3::new(0.0, 0.0, 0.5), rotation: Vec3::new(-90.0_f32.to_radians(), 0.0, 0.0)},
                TileType::East => Transformation {translation: Vec3::new(0.5, 0.0, 0.0), rotation: Vec3::new(0.0, 0.0, 90.0_f32.to_radians())},
                TileType::South => Transformation {translation: Vec3::new(0.0, 0.0, -0.5), rotation: Vec3::new(90.0_f32.to_radians(), 0.0, 0.0)},
                TileType::West => Transformation {translation: Vec3::new(-0.5, 0.0, 0.0), rotation: Vec3::new(0.0, 0.0, -90.0_f32.to_radians())},

                _ => Transformation {translation: Vec3::new(0.0, 0.0, 0.0), rotation: Vec3::new(0.0, 0.0, 0.0)}
            }
        )
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Rect3 {
    pub pos1: IVec3,
    pub pos2: IVec3,
}
impl Rect3 {
    pub fn new(pos: IVec3, width: i32, height: i32, length: i32) -> Rect3 {
        Rect3 {pos1: pos, pos2: IVec3::new(pos.x + width - 1, pos.y + height - 1, pos.z + length - 1)}
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect3) -> bool {
        let min_self = self.min();
        let max_self = self.min();

        let min_other = other.min();
        let max_other = other.max();

        min_self.x <= max_other.x && max_self.x >= min_other.x &&
        min_self.y <= max_other.y && max_self.y >= min_other.y &&
        min_self.z <= max_other.z && max_self.z >= min_other.z
    }

    pub fn center(&self) -> Vec3 { 
        Vec3::new((self.pos1.x + self.pos2.x) as f32 / 2.0, (self.pos1.y + self.pos2.y) as f32 /2.0, (self.pos1.z + self.pos2.z) as f32 / 2.0)
    }
    
    pub fn min(&self) -> IVec3 {
        IVec3::new(self.pos1.x.min(self.pos2.x), self.pos1.y.min(self.pos2.y), self.pos1.z.min(self.pos2.z))
    }

    pub fn max(&self) -> IVec3 {
        IVec3::new(self.pos1.x.max(self.pos2.x), self.pos1.y.max(self.pos2.y), self.pos1.z.max(self.pos2.z))
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tile {
    pub material: Handle<StandardMaterial>,
}

//#[derive(Default, Debug, Clone, PartialEq)]
//pub struct Tile {
//    pub Tiles: 
//}

#[derive(Clone)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub length: i32,
    pub rooms: Vec<Room>,
    pub tiles: Array3<EnumMap<TileType, Option<Tile>>>,
    pub tile_offsets: TileOffsets,
}
impl Default for Map {
    fn default() -> Map {
        Map {
            // Might need to add an "ID" type thingy once we start having more maps.
            // Not sure how we'll handle loading zones and such? Will figure it out tho prolly.
            width: 80,
            height: 10,
            length: 40,
            rooms: Vec::new(),
            tiles: Array3::<EnumMap<TileType, Option<Tile>>>::from_elem(
                (80, 10, 40),
                enum_map ! {
                    _ => None
                }
            ),
            tile_offsets: TileOffsets::default(),
        }
    }
}
impl Map {
    // Perhaps later we should make floor/walls/ceiling be options so we can create rects with open ends.
    /// Create abstract world from the geometric.
    /// This may make some changes to the geometric to ensure it fits within the abstract.
    fn update_tiles(&mut self) {
        for room in self.rooms.iter_mut() {
            if !room.spawned {
                match room.room_type {
                    RoomType::Rectangle(rect) => {
                        let min = rect.min();
                        let max = rect.max();

                        let mut area = self.tiles.slice_mut(
                            s![min.x as i32..=max.x as i32,
                               min.y as i32..=max.y as i32,
                               min.z as i32..=max.z as i32]
                        );
                
                        for ((x, y, z), tile) in area.indexed_iter_mut() {
                            tile.clear();
                
                            // Is there a better way to do this?
                            if y == (max.y - min.y) as usize {
                                tile[TileType::Ceiling] = Some(room.ceiling.clone());
                            }
                            if z == (max.z - min.z) as usize {
                                tile[TileType::North] = Some(room.walls.clone());
                            }
                            if x == (max.x - min.x) as usize {
                                tile[TileType::East] = Some(room.walls.clone());
                            }
                            // These are fine.
                            if y == 0 {
                                tile[TileType::Floor] = Some(room.floor.clone());
                            }
                            if z == 0 {
                                tile[TileType::South] = Some(room.walls.clone());
                            }
                            if x == 0 as usize {
                                tile[TileType::West] = Some(room.walls.clone());
                            }
                        }
                    }
                }
                room.spawned = true;
            }

            for entrance in room.entrances.iter_mut() {
                self.tiles[[entrance.location.x as usize, entrance.location.y as usize, entrance.location.z as usize]][entrance.orientation] = None;
            }

            for exit in room.exits.iter_mut() {
                match &mut exit.exit_type {
                    ExitType::Doorway { location, orientation, path, need_room, ceiling, walls, floor } => {
                        if *need_room == true {
                            let vector = path[path.len() - 1] - path[path.len() - 2];
                            let next = path[path.len() - 1] + vector;

                            if next.x < 0 || next.y < 0 || next.z < 0 || next.x > self.width - 1 || next.y > self.height - 1 || next.z > self.length - 1 {
                                // Uh oh, our path leads off map
                                // TODO: stuff

                                // Temp: just say we dont need a room lol
                                *need_room = false;
                            }
                            else {
                                let next_tile = &self.tiles[[next.x as usize, next.y as usize, next.z as usize]];

                                let mut empty = self.tiles[[0, 0, 0]].clone();
                                empty.clear();
    
                                if *next_tile == empty {
                                    if !exit.spawned {
                                        path.clear();
                                    }
                                }
                                else {
                                    *need_room = false;
                                }
                            }

                        }

                        if !exit.spawned {
                            let mut intersected = false;
                            self.tiles[[location.x as usize, location.y as usize, location.z as usize]][*orientation] = None;

                            'path_loop: for c in 0..path.len() {
                                let current = path[c];

                                if current.x < 0 || current.y < 0 || current.z < 0 || current.x > self.width - 1 || current.y > self.height - 1 || current.z > self.length - 1 {
                                    path.drain(c..path.len());
                                    break 'path_loop;
                                }

                                let current_tile = &mut self.tiles[[current.x as usize, current.y as usize, current.z as usize]];

                                let previous = 
                                    if c == 0 {
                                        let vec_change = self.tile_offsets.0[*orientation].translation * -2.0;
                                        let p = current + IVec3::new(vec_change.x as i32, vec_change.y as i32, vec_change.z as i32);
                                        p
                                    } else {
                                        path[c-1]
                                    };
                                let next = 
                                    if c == path.len() - 1 {
                                        current + current - previous
                                    } else {
                                        path[c+1]
                                    };
                                    

                                for tile in current_tile.iter() {
                                    if tile.1.is_some() {
                                        intersected = true;
                                        if current.z - 1 != previous.z {
                                            current_tile[TileType::North] = None;
                                        }
                                        if current.x - 1 != previous.x {
                                            current_tile[TileType::East] = None;
                                        }
                                        if current.z + 1 != previous.z {
                                            current_tile[TileType::South] = None;
                                        }
                                        if current.x + 1 != previous.x {
                                            current_tile[TileType::West] = None;
                                        }
                                        path.drain(c+1..path.len());
                                        break 'path_loop;
                                    }
                                }

                                current_tile[TileType::North] = Some(floor.clone());
                                current_tile[TileType::East] = Some(floor.clone());
                                current_tile[TileType::South] = Some(floor.clone());
                                current_tile[TileType::West] = Some(floor.clone());

                                if current.z + 1 == previous.z || current.z + 1 == next.z {
                                    current_tile[TileType::North] = None;
                                }
                                if current.x + 1 == previous.x || current.x + 1 == next.x {
                                    current_tile[TileType::East] = None;
                                }
                                if current.z - 1 == previous.z || current.z - 1 == next.z {
                                    current_tile[TileType::South] = None;
                                }
                                if current.x - 1 == previous.x || current.x - 1 == next.x {
                                    current_tile[TileType::West] = None;
                                }
                                
                                current_tile[TileType::Floor] = Some(floor.clone());
                                current_tile[TileType::Ceiling] = Some(floor.clone());
                            }
                            if !intersected {
                                *need_room = true;
                            }

                            exit.spawned = true;
                        }
                        


                    }

                    _ => {

                    }
                }



                

            }
        }
    }

    fn create_rect(&mut self, floor: Tile, walls: Tile, ceiling: Tile, rect: &Rect3) {
        let min = rect.min();
        let max = rect.max();

        let mut area = self.tiles.slice_mut(
            s![min.x as i32..=max.x as i32,
               min.y as i32..=max.y as i32,
               min.z as i32..=max.z as i32]
        );

        for ((x, y, z), tile) in area.indexed_iter_mut() {
            tile.clear();

            // Is there a better way to do this?
            if y == (max.y - min.y) as usize {
                tile[TileType::Ceiling] = Some(ceiling.clone());
            }
            if z == (max.z - min.z) as usize {
                tile[TileType::North] = Some(floor.clone());
            }
            if x == (max.x - min.x) as usize {
                tile[TileType::East] = Some(floor.clone());
            }
            // These are fine.
            if y == 0 {
                tile[TileType::Floor] = Some(floor.clone());
            }
            if z == 0 {
                tile[TileType::South] = Some(floor.clone());
            }
            if x == 0 as usize {
                tile[TileType::West] = Some(floor.clone());
            }
        }
    }

    fn rand_surface_wall_points(&self, min_points: i32, max_points: i32, rect: &Rect3) -> Vec<(IVec3, TileType)> {
        let mut rng = rand::thread_rng();
    
        let mut surface_wall_points = Vec::<IVec3>::new();
    
        let point_count = rng.gen_range(min_points..=max_points);
    
        let min = rect.min();
        let max = rect.max();

        for x in min.x..=max.x {
            surface_wall_points.push(IVec3::new(x, min.y, min.z));
            surface_wall_points.push(IVec3::new(x, min.y, max.z));
        }
        for z in min.z..=max.z {
            surface_wall_points.push(IVec3::new(min.x, min.y, z));
            surface_wall_points.push(IVec3::new(max.x, min.y, z));
        }
    
        let mut points = Vec::<(IVec3, TileType)>::new();
    
        for point in surface_wall_points.choose_multiple(&mut rng, point_count as usize) {
            let map_point = &self.tiles[[point.x as usize, point.y as usize, point.z as usize]];
            let mut possible_walls = Vec::new();
    
            // We should probably write a helper function for checking these...
            if map_point[TileType::North].is_some()  {
                possible_walls.push(TileType::North);
            }
            if map_point[TileType::East].is_some() {
                possible_walls.push(TileType::East);
            }
            if map_point[TileType::South].is_some() {
                possible_walls.push(TileType::South);
            }
            if map_point[TileType::West].is_some() {
                possible_walls.push(TileType::West);
            }


            points.push((*point, *possible_walls.choose(&mut rng).unwrap()));
        }
    
        points
    }

    /*
    pub fn rect_valid(&self, rect: Rect3) -> bool {
        rect.pos1.x > 0 || rect.pos1.y > 0 || rect.pos1.z > 0 ||
        rect.pos1.x < self.width - 1 || rect.pos1.y < self.height - 1 || rect.pos1.z < self.length - 1 ||
        rect.pos2.x > 0 || rect.pos2.y > 0 || rect.pos2.z > 0 ||
        rect.pos2.x < self.width - 1 || rect.pos2.y < self.height - 1 || rect.pos2.z < self.length - 1
    }
    
    pub fn rect_intersects(&self, rect: Rect3) -> bool {
        let min = rect.min();
        let max = rect.max();
        // Check other rects (They may not be spawned yet?)
        for room in &self.rooms {
            match room.room_type {
                RoomType::Rectangle(room_rect) => {
                    if rect.intersect(&room_rect) {
                        return true;
                    }
                }
            }
        }
        // Iterate over rects values to see if any collide with any spawned tiles.
        let mut empty = self.tiles[[0, 0, 0]].clone();
        empty.clear();
        
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=min.z {
                    let tile = &self.tiles[[x as usize, y as usize, z as usize]];
                    if *tile != empty {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    pub fn push_room_if_ok (&mut self, room: Room) -> Result<String, String> {
        match room.room_type {
            RoomType::Rectangle(rect) => {
                if !self.rect_valid(rect) {
                    Err("OOB".to_string())
                } 
                else if self.rect_intersects(rect) {
                    Err("Intersects".to_string())
                }
                else {
                    self.rooms.push(room);
                    Ok("Success".to_string())
                }
            }
        }
    }
     */
}

// Helper functions
/// Checks if a rect is not OOB

pub fn rect_valid(bounds: IVec3, rect: Rect3) -> bool {
    rect.pos1.x >= 0 && rect.pos1.y >= 0 && rect.pos1.z >= 0 &&
    rect.pos1.x < bounds.x - 1 && rect.pos1.y < bounds.y - 1 && rect.pos1.z < bounds.z - 1 &&
    rect.pos2.x >= 0 && rect.pos2.y >= 0 && rect.pos2.z >= 0 &&
    rect.pos2.x < bounds.x - 1 && rect.pos2.y < bounds.y - 1 && rect.pos2.z < bounds.z - 1
}

pub fn rect_intersects(map: &Map, rect: Rect3) -> bool {
    let min = rect.min();
    let max = rect.max();
    // Check other rects (They may not be spawned yet?)
    for room in &map.rooms {
        match room.room_type {
            RoomType::Rectangle(room_rect) => {
                if rect.intersect(&room_rect) {
                    return true;
                }
            }
        }
    }
    // Iterate over rects values to see if any collide with any spawned tiles.
    let mut empty = map.tiles[[0, 0, 0]].clone();
    empty.clear();
    
    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=min.z {
                let tile = &map.tiles[[x as usize, y as usize, z as usize]];
                if tile != &empty {
                    return true;
                }
            }
        }
    }
    return false;
}


#[derive(Clone, Debug)]
pub enum RoomType {
    Rectangle(Rect3),
}

#[derive(Clone, Debug)]
pub struct Room {
    pub spawned: bool,
    pub room_type: RoomType,
    pub exits: Vec<Exit>,
    pub entrances: Vec<Entrance>,
    pub ceiling: Tile,
    pub walls: Tile,
    pub floor: Tile,
}
impl Room {
    pub fn simple_rect (room: Rect3, exits: Vec<(IVec3, TileType)>, entrances: Vec<Entrance>, tile: Tile) -> Room {
        Room {
            spawned: false,
            room_type: RoomType::Rectangle(room),
            exits: exits.iter().map(|(pos, orientation)| Exit::empty_doorway(*pos, *orientation, tile.clone())).collect(),
            entrances: entrances,
            ceiling: tile.clone(),
            walls: tile.clone(),
            floor: tile.clone(),
        }
    }
    pub fn map_empty_doorways(&mut self, exits: Vec<(IVec3, TileType)>, tile: Tile) {
        self.exits = exits.iter().map(|(pos, orientation)| Exit::empty_doorway(*pos, *orientation, tile.clone())).collect();
    }
}

#[derive(Clone, Debug)]
pub enum ExitType {
    ClickWarp {
        location: IVec3,
        // This is going to need more data once we use it.
    },
    Doorway {
        location: IVec3,
        orientation: TileType,
        path: Vec<IVec3>,
        need_room: bool,
        ceiling: Tile,
        walls: Tile,
        floor: Tile,
    },
}


#[derive(Clone, Debug)]
pub struct Exit {
    spawned: bool,
    exit_type: ExitType,
}
impl Exit {
    pub fn empty_doorway (position: IVec3, orientation: TileType, tile: Tile) -> Exit {
        Exit {
            // We are lying here, but its a white lie because there's nothing to spawn yet.
            // It can be set to false later once it has an actual pathway somewhere.
            spawned: true,
            exit_type: ExitType::Doorway {location: position, 
                orientation: orientation, 
                path: Vec::new(),
                need_room: false,
                ceiling: tile.clone(), 
                walls: tile.clone(), 
                floor: tile.clone()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entrance {
    location: IVec3,
    orientation: TileType,
}
