use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

use crate::map::Tile;

//Plugin
#[derive(Default)]
pub struct AssetPlugin;
impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TileAssets>()
            .init_resource::<MaterialAssets>()
            .init_resource::<MeshAssets>();
    }
}

// For assets that we cannot load/create otherwise.
// Programatic shit basically I guess.
pub fn create_assets (
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_handles: Res<TextureAssets>,
    material_handles: Res<MaterialAssets>,
) {
    commands.insert_resource( MeshAssets {
        plane: meshes.add(Mesh::from(shape::Plane {size: 1.0}))
    });

    let grass_texture = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handles.grass.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });

    commands.insert_resource( MaterialAssets {
        grass: grass_texture.clone()
    });

    commands.insert_resource( TileAssets {
        grass: Tile {material: grass_texture.clone()},
    });
}

#[derive(AssetCollection)]
pub struct TextureAssets {
    #[asset(path = "textures/grass.png")]
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