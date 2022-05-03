use bevy::render::mesh::{VertexAttributeValues::*, Indices, PrimitiveTopology};
use bevy::{prelude::*, render::{mesh::VertexAttributeValues, render_resource::AddressMode}};
use iyes_loopless::state::NextState;

use crate::{map::Tile, GameState};

const TILING_SCALE: f32 = 1.0;

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
    mut textures: ResMut<Assets<Image>>,
    mut texture_assets: ResMut<TextureAssets>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    texture_assets.grass = asset_server.load("textures/grass.png");
    texture_assets.gray_medium_brick = asset_server.load("textures/gray_medium_brick.png");
    texture_assets.concrete = asset_server.load("textures/concrete.png");

    if let Some(gray_medium_brick_tex) = textures.get_mut(texture_assets.gray_medium_brick.clone()) {
        gray_medium_brick_tex.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        gray_medium_brick_tex.sampler_descriptor.address_mode_v = AddressMode::Repeat;
        gray_medium_brick_tex.sampler_descriptor.address_mode_w = AddressMode::Repeat;
    }
    else {
        return;
    }

    if let Some(concrete_tex) = textures.get_mut(texture_assets.concrete.clone()) {
        concrete_tex.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        concrete_tex.sampler_descriptor.address_mode_v = AddressMode::Repeat;
        concrete_tex.sampler_descriptor.address_mode_w = AddressMode::Repeat;
    }
    else {
        return;
    }

    let mut plane_mesh = Mesh::from(shape::Plane{size: 1.0});
    if let Some(VertexAttributeValues::Float32x2((uvs))) = 
       plane_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {

        //for uv in uvs {
        //    println!("uv: {:?}", uv);
        //    uv[0] *= TILING_SCALE;
        //    uv[1] *= TILING_SCALE;
        //}
    }

    let mut slab_mesh = box_no_squish(-0.5, 0.5, 0.0, 0.1, -0.5, 0.5);

    /*
    let mut slab_mesh = Mesh::from(shape::Box {
        min_x: -0.5,
        max_x: 0.5,
        min_y: 0.0,
        max_y: 0.1,
        min_z: -0.5,
        max_z: 0.5,
    });

    let temp_clone = slab_mesh.clone();
    let position_values = temp_clone.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    
    if let Some(Float32x2(uvs)) = slab_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        if let Float32x3(positions) = position_values {
            //iterate over every face
            for i in (0..uvs.len()).step_by(4) {
                println!("epic positions of a face:");
                
                println!("uv: {:?}. position: {:?}", uvs[i], positions[i]);
                println!("uv: {:?}. position: {:?}", uvs[i+1], positions[i+1]);
                println!("uv: {:?}. position: {:?}", uvs[i+2], positions[i+2]);
                println!("uv: {:?}. position: {:?}", uvs[i+3], positions[i+3]);
            }


            /*
            for (i, uv) in uvs.iter_mut().enumerate() {
                println!("uv: {:?}. position: {:?}", uv, positions[i]);
                uv[0] *= TILING_SCALE;
                uv[1] *= TILING_SCALE * positions[i][0];
            }
             */
        }
     

    }
    */

    let plane = meshes.add(plane_mesh);

    let slab = meshes.add(slab_mesh);

    commands.insert_resource( MeshAssets {
        plane: plane.clone()
    });

    let grass_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_assets.grass.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });

    let brick_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_assets.gray_medium_brick.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });

    let concrete_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_assets.concrete.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..Default::default()
    });

    commands.insert_resource( MaterialAssets {
        grass: grass_material.clone(),
        gray_medium_brick: brick_material.clone(),
        concrete: concrete_material.clone(),
    });

    commands.insert_resource( TileAssets {
        grass: Tile {mesh: plane.clone(), material: grass_material.clone()},
        gray_medium_brick: Tile {mesh: slab.clone(), material: brick_material.clone()},
        concrete: Tile {mesh: plane.clone(), material: concrete_material.clone()},
    });

    commands.insert_resource(NextState(GameState::StartMapGen));
}


// Helper Functions
pub fn box_no_squish (min_x: f32, max_x: f32, min_y: f32, max_y: f32, min_z: f32, max_z: f32) -> Mesh {
    let vertices = &[
        // Top
        ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
        ([max_x, min_y, max_z], [0., 0., 1.0], [max_x-min_x, 0.0]),
        ([max_x, max_y, max_z], [0., 0., 1.0], [max_x-min_x, max_y-min_y]),
        ([min_x, max_y, max_z], [0., 0., 1.0], [0., max_y-min_y]),
        // Bottom
        ([min_x, max_y, min_z], [0., 0., -1.0], [max_x-min_x, 0.]),
        ([max_x, max_y, min_z], [0., 0., -1.0], [0., 0.]),
        ([max_x, min_y, min_z], [0., 0., -1.0], [0., max_y-min_y]),
        ([min_x, min_y, min_z], [0., 0., -1.0], [max_x-min_x, max_y-min_y]),
        // Right
        ([max_x, min_y, min_z], [1.0, 0., 0.], [0., 0.]),
        ([max_x, max_y, min_z], [1.0, 0., 0.], [max_y-min_y, 0.]),
        ([max_x, max_y, max_z], [1.0, 0., 0.], [max_y-min_y, max_z-min_z]),
        ([max_x, min_y, max_z], [1.0, 0., 0.], [0., max_z-min_z]),
        // Left
        ([min_x, min_y, max_z], [-1.0, 0., 0.], [max_y-min_y, 0.]),
        ([min_x, max_y, max_z], [-1.0, 0., 0.], [0., 0.]),
        ([min_x, max_y, min_z], [-1.0, 0., 0.], [0., max_z-min_z]),
        ([min_x, min_y, min_z], [-1.0, 0., 0.], [max_y-min_y, max_z-min_z]),
        // Front
        ([max_x, max_y, min_z], [0., 1.0, 0.], [max_x-min_x, 0.]),
        ([min_x, max_y, min_z], [0., 1.0, 0.], [0., 0.]),
        ([min_x, max_y, max_z], [0., 1.0, 0.], [0., max_z-min_z]),
        ([max_x, max_y, max_z], [0., 1.0, 0.], [max_x-min_x, max_z-min_z]),
        // Back
        ([max_x, min_y, max_z], [0., -1.0, 0.], [0., 0.]),
        ([min_x, min_y, max_z], [0., -1.0, 0.], [max_x-min_x, 0.]),
        ([min_x, min_y, min_z], [0., -1.0, 0.], [max_x-min_x, max_z-min_z]),
        ([max_x, min_y, min_z], [0., -1.0, 0.], [0., max_z-min_z]),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);

    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}

// Do we even need this???
pub fn get_locked_axis(positions: Vec<[f32; 3]>, index: usize, vertices: usize) -> Result<usize, String> {
    'axis_iter: for (axis, _) in positions[0].iter().enumerate() {
        let mut last_val = positions[index];
        'value_check: for i in index..index+vertices {
            if last_val[axis] != positions[i][axis] {
                continue 'axis_iter;
            }
            last_val = positions[index]
        }
        return Ok(axis);
    }

    Err("No locked axis!".to_string())
}

// Resources
#[derive(Default)]
pub struct TextureAssets {
    pub grass: Handle<Image>,
    pub gray_medium_brick: Handle<Image>,
    pub concrete: Handle<Image>,
}

#[derive(Default)]
pub struct MaterialAssets {
    pub grass: Handle<StandardMaterial>,
    pub gray_medium_brick: Handle<StandardMaterial>,
    pub concrete: Handle<StandardMaterial>,
}

#[derive(Default)]
pub struct MeshAssets {
    pub plane: Handle<Mesh>,
}

#[derive(Default)]
pub struct TileAssets {
    pub grass: Tile,
    pub gray_medium_brick: Tile,
    pub concrete: Tile,
}