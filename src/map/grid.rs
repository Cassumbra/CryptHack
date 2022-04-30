use std::ops::{Index, IndexMut};

use bevy::prelude::*;
use enum_map::{EnumMap, Enum, enum_map};
use heron::{CollisionShape, RigidBody, CollisionLayers};
use ndarray::{Array3, Axis};

use super::{geometric::Tile, WithinBoxIterator};


// Helper Systems
pub fn clear_position ( commands: &mut Commands, map: &mut GridMap, position: IVec3) {
    for (_tile, opt_entity) in map[position] {
        if let Some(entity) = opt_entity {
            commands.entity(entity).despawn();
        }
    }

    map[position] = TileType::empty();
}

pub fn clear_tile ( commands: &mut Commands, map: &mut GridMap, tile_type: TileType, position: IVec3) {
    if let Some(entity) = map[position][tile_type] {
        commands.entity(entity).despawn_recursive();
    }

    map[position][tile_type] = None;
}

pub fn spawn_tile ( commands: &mut Commands, map: &mut GridMap, tile: Tile, tile_type: TileType, position: IVec3) {
    let transformation = TileOffsets::default()[tile_type];
    let mut transform = Transform::default();

    transform.translation = Vec3::new(position.x as f32, position.y as f32, position.z as f32) + transformation.translation;

    if transformation.rotation.x != 0.0 {
        transform.rotate(Quat::from_rotation_x(transformation.rotation.x))
    }
    if transformation.rotation.y != 0.0 {
        transform.rotate(Quat::from_rotation_y(transformation.rotation.y))
    }
    if transformation.rotation.z != 0.0 {
        transform.rotate(Quat::from_rotation_z(transformation.rotation.z))
    }

    let spawned_tile = commands
        .spawn_bundle(PbrBundle {
            mesh: tile.mesh.clone(),
            material: tile.material.clone(),
            ..Default::default()    
        })  
        .insert(transform)
        .insert(GlobalTransform::default())
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.5, 0.0, 0.5),
            border_radius: None,
        })
        .insert(RigidBody::Static)
        .insert(CollisionLayers::default())
        .id();

    map[position][tile_type] = Some(spawned_tile);
}



// Resources
#[derive(Clone, Deref, DerefMut)]
pub struct GridMap (Array3<EnumMap<TileType, Option<Entity>>>);
impl GridMap {
    pub fn width(&self) -> i32 {
        self.len_of(Axis(0)) as i32
    }
    pub fn height(&self) -> i32 {
        self.len_of(Axis(1)) as i32
    }
    pub fn length(&self) -> i32 {
        self.len_of(Axis(2)) as i32
    }

    pub fn min(&self) -> IVec3 {
        IVec3::new(0, 0, 0)
    }
    pub fn max(&self) -> IVec3 {
        IVec3::new(self.width() - 1, self.height() - 1, self.length() - 1)
    }

    pub fn position_oob(&self, position: IVec3) -> bool {
        let min = self.min();
        let max = self.max();

        position.x < min.x || position.y < min.y || position.z < min.z ||
        position.x > max.x || position.y > max.y || position.z > max.z
    }

    pub fn position_collides(&self, position: IVec3) -> bool {
        self[position] != TileType::empty()
    }
}
impl Default for GridMap {
    fn default() -> Self {
        GridMap(Array3::<EnumMap<TileType, Option<Entity>>>::from_elem(
            (80, 10, 40),
            enum_map ! {
                _ => None
            }
        ),)
    }
}
impl Index<IVec3> for GridMap {
    type Output = EnumMap<TileType, Option<Entity>>;

    fn index(&self, index: IVec3) -> &Self::Output {
        &self.0[[index.x as usize, index.y as usize, index.z as usize]]
    }
}
impl IndexMut<IVec3> for GridMap {
    fn index_mut(&mut self, index: IVec3) -> &mut Self::Output {
        &mut self.0[[index.x as usize, index.y as usize, index.z as usize]]
    }
}
impl IntoIterator for GridMap {
    type Item = IVec3;

    type IntoIter = WithinBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        WithinBoxIterator::new(self.min(), self.max())
    }
}
impl IntoIterator for &GridMap {
    type Item = IVec3;

    type IntoIter = WithinBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        WithinBoxIterator::new(self.min(), self.max())
    }
}

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
    pub fn rotate90(&self, left: bool) -> TileType {
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

    pub fn empty() -> EnumMap<TileType, Option<Entity>> {
        EnumMap::<TileType, Option<Entity>>::default()
    }
}
impl Default for TileType {
    fn default() -> Self {
        TileType::Center
    }
}

#[derive(Default, Clone, Copy)]
pub struct Transformation {
    pub translation: Vec3,
    pub rotation: Vec3,
}
impl Transformation {
    fn with_translation(translate: Vec3) -> Transformation {
        Transformation {
            translation: translate,
            rotation: Vec3::default(),
        }
    }
}

#[derive(Clone, Copy, Deref)]
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