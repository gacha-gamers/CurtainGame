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

use super::BulletContainer;

pub struct BulletRenderPlugin;

impl Plugin for BulletRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<BulletMaterial>::default())
            .init_resource::<BulletMeshes>()
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
struct BulletMeshes(Vec<Handle<Mesh>>);

fn make_bullet_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<BulletMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut bullet_meshes: ResMut<BulletMeshes>,
    container: Res<BulletContainer>,
    asset_server: Res<AssetServer>,
    mut last_mesh_count: Local<usize>,
) {
    #[cfg(trace_tracy)]
    let _span = info_span!("bullet mesh processing").entered();

    // Collect all bullets that are not dead
    let bullets_to_render: Vec<(Vec2, f32)> =
        (&container.positions, &container.rotations, &container.ages)
            .into_par_iter()
            .filter_map(|(position, rotation, age)| {
                BulletContainer::is_alive(*age).then_some((*position, *rotation))
            })
            .collect();

    // Iterate through the bullets' positions and rotations,
    // zipped together, in parallel, split into chunks of 4096 bullets.
    // Each chunk of bullets gets converted into a mesh, which then get used by MaterialMesh2d renderers below.
    let mut generated_meshes: Vec<Mesh> = bullets_to_render
        .par_chunks(CHUNK_SIZE)
        .map(|chunk| make_chunk_mesh(chunk, Vec2::new(8., 14.)))
        .collect();

    // If last frame, there were more meshes than this frame, ensure that those extra meshes get replaced by empty meshes (clear old bullets)
    let new_mesh_count = generated_meshes.len();
    generated_meshes
        .extend((0..last_mesh_count.saturating_sub(new_mesh_count)).map(|_| make_empty_mesh()));
    *last_mesh_count = new_mesh_count;

    #[cfg(trace_tracy)]
    event!(Level::INFO, "bullet mesh - creation done");

    for (i, generated_mesh) in generated_meshes.into_iter().enumerate() {
        // This is literally the only time if/else I wrote in the entire codebase so far. Refreshingly simple!

        // If there aren't enough bullet MaterialMesh2d renderers yet, spawn a new one after adding the mesh asset to the Assets
        if bullet_meshes.0.len() <= i {
            println!("Created chunk: {i}");
            bullet_meshes.0.push(mesh_assets.add(generated_mesh));

            // Spawn bullet MaterialMesh2d renderer with the new, just-added handle
            commands.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(bullet_meshes.0[i].clone()),
                material: materials.add(BulletMaterial {
                    color_texture: asset_server.load("SA_bullet.png"),
                }),
                ..Default::default()
            });
        } else {
            // If there is already a mesh renderer, just update the mesh in the Assets by setting it to the new generated mesh
            bullet_meshes.0[i] = mesh_assets.set(bullet_meshes.0[i].clone(), generated_mesh);
        }
    }
}

fn make_chunk_mesh(positions: &[(Vec2, f32)], size: Vec2) -> Mesh {
    let w = size.x / 2.0;
    let h = size.y / 2.0;

    #[cfg(trace_tracy)]
    let _span = info_span!("bullet mesh chunk generation").entered();

    /* There's no need to send UVs anymore: they're calculated in the shader */
    // let uv_config = [[0., 0.0], [1., 0.0], [0., 1.0], [1., 1.0]];
    // let uvs = uv_config.repeat(bullets.len());

    /* There's no need to generate indices anymore: see below for updated index code */
    // let mut indices = vec![0; positions.len() * 6];
    // indices.copy_from_slice(&TOTAL_INDICES[0..positions.len() * 6]);

    #[cfg(trace_tracy)]
    event!(Level::INFO, "bullet mesh chunk - indices done");

    let positions: Vec<Vec2> = positions
        .into_iter()
        .flat_map(|(p, rotation)| {
            let (sin, cos) = (rotation - PI / 2.).sin_cos();
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
        .collect();

    #[cfg(trace_tracy)]
    event!(Level::INFO, "bullet mesh chunk - positions done");

    // Always generates indices for all 4096 bullets in a chunk.
    // From testing, this was *way* faster than generating the indices,
    // and about as fast as copying only the slice of the total indices that is necessary.
    let indices = Indices::U32(TOTAL_INDICES.to_vec());

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(ATTRIBUTE_BULLET_POSITION, positions);
    mesh
}

fn make_empty_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(ATTRIBUTE_BULLET_POSITION, Vec::<Vec2>::new());
    mesh.set_indices(None);
    mesh
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "73085f16-c574-4e05-bbda-b75306a44ff9"]
pub struct BulletMaterial {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Handle<Image>,
}

impl Material2d for BulletMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bullet_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/bullet_material.wgsl".into()
    }
}
