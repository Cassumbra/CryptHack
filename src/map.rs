use bevy::{prelude::*, render::render_resource::Texture};
use enum_map::{EnumMap, Enum, enum_map};
use heron::prelude::*;
use rand::Rng;
use ndarray::{Array3, s};

use super::TextureAssets;

//Plugin
#[derive(Default)]
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<TileOffsets>()
        .init_resource::<Map>()
        .add_event::<SpawnSurfaceEvent>()
        .add_event::<SpawnSurfacesEvent>();
    }
}

// Systems
pub fn generate_map (
    mut ev_spawn_surface: EventWriter<SpawnSurfaceEvent>,
    mut ev_spawn_surfaces: EventWriter<SpawnSurfacesEvent>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<Map>,
    texture_handles: Res<TextureAssets>,
) {
    // We should maybe not be creating these here. Check bevy_asset_loader to see if there is a better way.
    let plane_mesh = meshes.add(Mesh::from(shape::Plane {size: 1.0}));
    let grass_texture = materials.add( StandardMaterial {
        base_color_texture: Some(texture_handles.grass.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });
    let grass_tile = Tile {material: grass_texture.clone()};

    //
    //let mut map_objects: Grid<Option<Entity>> = Grid::default([map_size.width, map_size.height]);

    let mut rng = rand::thread_rng();

    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;


    let mut rooms = Vec::<Rect3>::new();

    for _i in 0..=MAX_ROOMS {
        let w = rng.gen_range(MIN_SIZE..MAX_SIZE) as f32;
        let h = 0.0;
        let l = rng.gen_range(MIN_SIZE..MAX_SIZE) as f32;
        let x = rng.gen_range(1.0..(map.width - w - 1.0));
        let y = 0.0;//map.height;
        let z = rng.gen_range(1.0..(map.length - l - 1.0));

        let room = Rect3::new(Vec3::new(x, y, z), w, h, l);
        
        let mut ok = true;

        for other_room in rooms.iter() {
            if room.intersect(other_room) { ok = false }
        }
        if ok {
            rooms.push(room);
            map.create_rect(grass_tile.clone(), grass_tile.clone(), grass_tile.clone(), &room);
        }
    }

    for ((x, y, z), section) in map.tiles.indexed_iter() {
        for (tile_type, opt_tile) in section.iter() {
            if let Some(tile) = opt_tile {
                ev_spawn_surface.send(SpawnSurfaceEvent { material: tile.material.clone(), mesh: plane_mesh.clone(), location: Vec3::new(x as f32, y as f32, z as f32), tile_type: tile_type });
            }
        }
    }

    //for room in rooms.iter() {
    //    ev_spawn_surfaces.send(SpawnSurfacesEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location1: room.pos1, location2: room.pos2, tile_type: TileType::Floor });
    //}

    
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(0.0, 0.0, 0.0), tile_type: TileType::Floor });
    /*
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(1.0, 0.0, 0.0), tile_type: TileType::Floor });
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(1.0, 0.0, 0.0), tile_type: TileType::East });

    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(0.0, 2.0, 3.0), tile_type: TileType::North });
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(0.0, 2.0, -3.0), tile_type: TileType::South });
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(3.0, 2.0, 0.0), tile_type: TileType::East });
    ev_spawn_surface.send(SpawnSurfaceEvent { material: grass_texture.clone(), mesh: plane_mesh.clone(), location: Vec3::new(-3.0, 2.0, 0.0), tile_type: TileType::West });
     */
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

// Events
pub struct SpawnSurfaceEvent{
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    location: Vec3,
    tile_type: TileType,
}

pub struct SpawnSurfacesEvent{
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    location1: Vec3,
    location2: Vec3,
    tile_type: TileType,
}

// Data
#[derive(Enum, Clone, Copy)]
pub enum TileType {
    Center,
    Ceiling,
    Floor,
    North,
    East,
    South,
    West,
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

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Rect3 {
    pub pos1: Vec3,
    pub pos2: Vec3,
}
impl Rect3 {
    pub fn new(pos: Vec3, width: f32, height: f32, length: f32) -> Rect3 {
        Rect3 {pos1: pos, pos2: Vec3::new(pos.x + width, pos.y + height, pos.z + length)}
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect3) -> bool {
        self.pos1.x <= other.pos2.x && self.pos2.x >= other.pos1.x &&
        self.pos1.y <= other.pos2.y && self.pos2.y >= other.pos1.y &&
        self.pos1.z <= other.pos2.z && self.pos2.z >= other.pos1.z
    }

    pub fn center(&self) -> Vec3 { 
        Vec3::new((self.pos1.x + self.pos2.x)/2.0, (self.pos1.y + self.pos2.y)/2.0, (self.pos1.z + self.pos2.z)/2.0)
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Tile {
    pub material: Handle<StandardMaterial>,
}

pub struct Map {
    pub width: f32,
    pub height: f32,
    pub length: f32,
    pub tiles: Array3<EnumMap<TileType, Option<Tile>>>,
}
impl Default for Map {
    fn default() -> Map {
        Map {
            // Might need to add an "ID" type thingy once we start having more maps.
            // Not sure how we'll handle loading zones and such? Will figure it out tho prolly.
            width: 80.0,
            height: 1.0,
            length: 40.0,
            tiles: Array3::<EnumMap<TileType, Option<Tile>>>::from_elem(
                (80, 1, 40),
                enum_map ! {
                    _ => None
                }
            ),
        }
    }
}
impl Map {
    // Perhaps later we should make floor/walls/ceiling be options so we can create rects with open ends.
    fn create_rect(&mut self, floor: Tile, walls: Tile, ceiling: Tile, rect: &Rect3) {
        let mut area = self.tiles.slice_mut(
            s![rect.pos1.x as i32..=rect.pos2.x as i32,
               rect.pos1.y as i32..=rect.pos2.y as i32,
               rect.pos1.z as i32..=rect.pos2.z as i32]
        );

        // TODO: We need to wipe things from the map if it already has stuff there. (I think??? I'm really tired ;-;)
        for ((x, y, z), tile) in area.indexed_iter_mut() {
            // We should be checking if its actually floor level instead of assuming it is.
            tile[TileType::Floor] = Some(floor.clone());

            // Is there a better way to do this?
            if z == (rect.pos2.z - rect.pos1.z) as usize {
                tile[TileType::North] = Some(floor.clone());
            }
            if x == (rect.pos2.x - rect.pos1.x) as usize {
                tile[TileType::East] = Some(floor.clone());
            }
            // These are fine.
            if z == 0 {
                tile[TileType::South] = Some(floor.clone());
            }
            if x == 0 as usize {
                tile[TileType::West] = Some(floor.clone());
            }

            // We should be checking if its actually ceiling level instead of assuming it is.
            tile[TileType::Ceiling] = Some(ceiling.clone());
        }
    }
}