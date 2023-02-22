use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        once_cell::sync::Lazy,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef, VertexFormat},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use rayon::prelude::*;

#[cfg(feature = "trace")]
use bevy::{log::Level, utils::tracing::event};

use super::BulletPool;

pub struct BulletRenderPlugin;

impl Plugin for BulletRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<BulletChunkMaterial>::default())
            .init_resource::<BulletChunkHandles>()
            .add_startup_system(init_bullet_material)
            .add_system(make_bullet_meshes);
    }
}

const CHUNK_SIZE: usize = 4096;

// Custom vertex attribute for passing Vec2's instead of Vec3's in the meshes for the Vertex Position
pub const ATTRIBUTE_BULLET_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Bullet_Vertex_Position", 0, VertexFormat::Float32x2);

// Pre-calculated array of indices, since those are exactly the same for every mesh.
static TOTAL_INDICES: Lazy<[u32; CHUNK_SIZE * 6]> = Lazy::new(|| {
    let mut total_indices = [0u32; CHUNK_SIZE * 6];
    for i in 0..CHUNK_SIZE {
        let slice_i = i * 6;
        let slice = &mut total_indices[slice_i..slice_i + 6];
        let n = (i * 4) as u32;

        // Calculate all the indices following the pattern: [0, 2, 1, 2, 3, 1] + n
        // According to the `impl From<Quad> for Mesh` function (slightly modified)
        slice.copy_from_slice(&[n, n + 2, n + 1, n + 2, n + 3, n + 1]);
    }
    total_indices
});

#[derive(Default, Resource)]
struct BulletChunkHandles {
    meshes: Vec<Handle<Mesh>>,
    material: Handle<BulletChunkMaterial>,
}

fn init_bullet_material(
    mut materials: ResMut<Assets<BulletChunkMaterial>>,
    mut chunk_handles: ResMut<BulletChunkHandles>,
    asset_server: Res<AssetServer>,
) {
    chunk_handles.material = materials.add(BulletChunkMaterial {
        color_texture: asset_server.load("SA_bullet.png"),
    });
}

fn make_bullet_meshes(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut chunk_handles: ResMut<BulletChunkHandles>,
    container: Res<BulletPool>,
    mut last_mesh_count: Local<usize>,
) {
    let bullets_to_render = get_alive_bullets(container);

    let chunks_to_render = (bullets_to_render.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    // Spawn new chunks if there are more chunks to render than the current amount of chunks
    for _ in chunk_handles.meshes.len()..chunks_to_render {
        spawn_chunk(&mut commands, &mut mesh_assets, &mut chunk_handles);
    }

    // The code in this function is terrible, but I have to deal with it to find and sort the meshes to update.
    // It wouldn't be so bad if I didn't need Mesh assets to store my vertices...
    let meshes = get_chunk_meshes(chunk_handles, &mut mesh_assets);

    // Ensure that chunks which no longer have any bullets will still be cleared
    let chunks_to_clear = (chunks_to_render..*last_mesh_count)
        .into_par_iter()
        .map(|_| -> Vec<Vec2> { vec![] });
    
    update_mesh_vertices(bullets_to_render, chunks_to_clear, meshes);

    *last_mesh_count = chunks_to_render;

    #[cfg(feature = "trace")]
    event!(Level::INFO, "bullet mesh - creation done");
}

fn get_alive_bullets(container: Res<BulletPool>) -> Vec<(Vec2, f32)> {
    let _span = info_span!("bullet_zipping").entered();
    container
        .positions
        .iter()
        .zip(container.rotations.iter())
        .zip(container.ages.iter())
        .filter_map(|((pos, rot), age)| BulletPool::is_alive(*age).then_some((*pos, *rot)))
        .collect()
}

fn spawn_chunk(
    commands: &mut Commands,
    mesh_assets: &mut ResMut<Assets<Mesh>>,
    chunk_handles: &mut ResMut<BulletChunkHandles>,
) {
    fn generate_empty_mesh() -> Mesh {    
        /* There's no need to send UVs anymore: they're calculated in the shader */
        // let uv_config = [[0., 0.0], [1., 0.0], [0., 1.0], [1., 1.0]];
        // let uvs = uv_config.repeat(bullets.len());

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(ATTRIBUTE_BULLET_POSITION, Vec::<Vec2>::new());
        mesh.set_indices(Some(Indices::U32(TOTAL_INDICES.to_vec())));
        mesh
    }

    let handle = mesh_assets.add(generate_empty_mesh());

    chunk_handles.meshes.push(handle.clone());
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(handle.clone()),
        material: chunk_handles.material.clone(),
        ..Default::default()
    });
}

fn update_mesh_vertices<T: IndexedParallelIterator<Item = Vec<Vec2>>>(
    bullets_to_render: Vec<(Vec2, f32)>,
    chunks_to_clear: T,
    meshes: Vec<(usize, &mut Mesh)>,
) {
    let _span = info_span!("mesh_updating").entered();
    // Iterate through the bullets' positions and rotations,
    // zipped together, in parallel, split into chunks of bullets.
    // Each chunk produces a vector of vertex positions for each corresponding chunk mesh.
    bullets_to_render
        .par_chunks(CHUNK_SIZE)
        .map(|chunk| generate_vertices(chunk, Vec2::new(8., 14.)))
        .chain(chunks_to_clear)
        .zip(meshes.into_par_iter())
        .for_each(|(positions, (_, mesh))| {
            mesh.insert_attribute(ATTRIBUTE_BULLET_POSITION, positions);
        });
}

fn get_chunk_meshes<'a>(
    chunk_handles: ResMut<BulletChunkHandles>,
    mesh_assets: &'a mut ResMut<Assets<Mesh>>,
) -> Vec<(usize, &'a mut Mesh)> {
    let mesh_ids: Vec<_> = chunk_handles.meshes.iter().map(|h| h.id()).collect();

    let mut meshes: Vec<_> = mesh_assets
        .iter_mut()
        .filter_map(|(asset, mesh_ref)| {
            let index = mesh_ids.iter().position(|i| *i == asset);
            index.and_then(|i| Some((i, mesh_ref)))
        })
        .collect();
    meshes.sort_by_key(|a| a.0);
    meshes
}

fn generate_vertices(positions: &[(Vec2, f32)], size: Vec2) -> Vec<Vec2> {
    #[cfg(feature = "trace")]
    let _span = info_span!("bullet_chunk_generation").entered();

    let w = size.x / 2.0;
    let h = size.y / 2.0;

    positions
        .into_iter()
        .flat_map(|(p, rotation)| {
            let (sin, cos) = (*rotation - PI / 2.).sin_cos();
            let base_a = Vec2::new(cos * w + sin * h, sin * w - cos * h);
            let base_b = Vec2::new(cos * w - sin * h, sin * w + cos * h);

            [*p - base_a, *p + base_b, *p - base_b, *p + base_a]

            // Expressions originated from trigonometry:
            // x = (cos a, sin a) * w
            // y = (-sin a, cos a) * h
            // vertex_1 = *position - base_a; // -x + y = (-cw - sh, -sw + ch) = -base_a | UV = [0, 0]
            // vertex_2 = *position + base_b; // +x + y = (+cw - sh, +sw + ch) = +base_b | UV = [1, 0]
            // vertex_3 = *position - base_b; // -x - y = (-cw + sh, -sw - ch) = -base_b | UV = [0, 1]
            // vertex_4 = *position + base_a; // +x - y = (+cw + sh, +sw - ch) = +base_a | UV = [1, 1]
        })
        .collect()
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "73085f16-c574-4e05-bbda-b75306a44ff9"]
pub struct BulletChunkMaterial {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Handle<Image>,
}

impl Material2d for BulletChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bullet_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/bullet_material.wgsl".into()
    }
}
