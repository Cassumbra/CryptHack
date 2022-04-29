use bevy::prelude::*;
use iyes_loopless::state::NextState;

use crate::{map::Tile, GameState};

//Plugin
#[derive(Default)]
pub struct AssetPlugin;
impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TextureAssets>()
            .init_resource::<TileAssets>()
            .init_resource::<MaterialAssets>()
            .init_resource::<MeshAssets>();
    }
}

// For assets that we cannot load/create otherwise.
// Programatic shit basically I guess.
pub fn create_assets (
    mut commands: Commands,

    asset_server: Res<AssetServer>,
    mut texture_assets: ResMut<TextureAssets>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    texture_assets.grass = asset_server.load("textures/grass.png");


    let plane = meshes.add(Mesh::from(shape::Plane {size: 1.0}));

    commands.insert_resource( MeshAssets {
        plane: plane.clone()
    });

    let grass_texture = materials.add(StandardMaterial {
        base_color_texture: Some(texture_assets.grass.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });

    commands.insert_resource( MaterialAssets {
        grass: grass_texture.clone()
    });

    commands.insert_resource( TileAssets {
        grass: Tile {mesh: plane.clone(), material: grass_texture.clone()},
    });

    commands.insert_resource(NextState(GameState::StartMapGen));
}

#[derive(Default)]
pub struct TextureAssets {
    pub grass: Handle<Image>,
}

#[derive(Default)]
pub struct MaterialAssets {
    pub grass: Handle<StandardMaterial>,
}

#[derive(Default)]
pub struct MeshAssets {
    pub plane: Handle<Mesh>,
}

#[derive(Default)]
pub struct TileAssets {
    pub grass: Tile,
}