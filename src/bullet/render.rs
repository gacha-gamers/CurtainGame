use std::f32::consts::PI;
// use bevy_tasks::prelude::*;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        once_cell::sync::Lazy,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    tasks::ComputeTaskPool,
};
use itertools::Itertools;

use super::BulletContainer;

pub struct BulletRenderPlugin;
const CHUNK_SIZE: usize = 4096;

impl Plugin for BulletRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<CustomMaterial>::default())
            .init_resource::<BulletMesh>()
            .add_system(make_bullet_mesh);
    }
}

#[derive(Default, Resource)]
struct BulletMesh(Vec<Handle<Mesh>>);

/* fn something_in_the_sky(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut bullet_mesh: ResMut<BulletMesh>,
    asset_server: Res<AssetServer>,
) {
    let mesh = mesh_assets.add(make_mesh(
        vec![Vec2::new(30., 10.), Vec2::new(20., -10.)],
        Vec2::new(8., 14.),
    ));

    bullet_mesh.0 = mesh;

    commands.spawn(MaterialMesh2dBundle {
        material: materials.add(CustomMaterial {
            color: Color::RED,
            color_texture: asset_server.load("SA_bullet.png"),
        }),
        mesh: bullet_mesh.0.clone().into(),
        ..Default::default()
    });
} */

static TOTAL_INDICES: Lazy<[u32; CHUNK_SIZE * 6]> = Lazy::new(|| {
    let mut total_indices = [0u32; CHUNK_SIZE * 6];
    for i in 0..CHUNK_SIZE {
        let slice_i = i * 6;
        let slice = &mut total_indices[slice_i..slice_i + 6];
        let i = (i * 4) as u32;
        slice.copy_from_slice(&[i, i + 2, i + 1, i + 2, i + 3, i + 1]);
    }
    total_indices
});

fn make_bullet_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut bullet_mesh: ResMut<BulletMesh>,
    container: Res<BulletContainer>,
    asset_server: Res<AssetServer>,
) {
    // let chunk_count = (bullet_query.iter().len() + CHUNK_SIZE - 1) / CHUNK_SIZE;

    /* let bullets_vec = bullet_query.iter().collect_vec();
    let slice: &[&Bullet] = bullets_vec.as_ref();

    let meshes = ComputeTaskPool::get().scope(|scope| {
        for chunk in slice.chunks(CHUNK_SIZE) {
            scope.spawn(async move { make_mesh(chunk, Vec2::new(8., 14.)) });
        }
    }); */
    //let _span = info_span!("bullet mesh processing").entered();

    let positions = container.positions.clone();
    let rotations = container.rotations.clone();

    //event!(Level::INFO, "bullet mesh - cloning done");
    /*
    let meshes = positions
        .into_iter()
        .zip(rotations)
        .chunks(CHUNK_SIZE)
        .into_iter()
        .map(|chunk| {
            make_mesh(chunk, Vec2::new(8., 14.))
        })
        .collect_vec(); */

    // let slice: Zip<IntoIter<Vec2>, IntoIter<f32>> = positions.clone().into_iter().zip(rotations.clone());
    let bullets_vec = positions.into_iter().zip(rotations).collect_vec();
    let slice: &[(Vec2, f32)] = bullets_vec.as_ref();

    let meshes = ComputeTaskPool::get().scope(|scope| {
        for chunk in slice.chunks(CHUNK_SIZE) {
            scope.spawn(async move { make_mesh(chunk, Vec2::new(8., 14.)) });
        }
    });

    //event!(Level::INFO, "bullet mesh - creation done");

    /*
    let meshes = ComputeTaskPool::get().scope(|scope| {
        for (i, chunk) in bullets_chunks.into_iter().enumerate() {
            scope.spawn(async move {  });
        }
    }); */
    // bullet_query.par_for_each(batch_size, f)

    for (i, new_mesh) in meshes.into_iter().enumerate() {
        // TODO: Make this code less hideous
        // This is literally the only time if/else that I wrote in the entire codebase so far TvT
        if bullet_mesh.0.len() <= i {
            println!("Created chunk: {i}");
            bullet_mesh.0.push(mesh_assets.add(new_mesh));
            commands.spawn(MaterialMesh2dBundle {
                material: materials.add(CustomMaterial {
                    color: Color::RED,
                    color_texture: asset_server.load("SA_bullet.png"),
                }),
                mesh: Mesh2dHandle(bullet_mesh.0[i].clone()),
                ..Default::default()
            });
        } else {
            bullet_mesh.0[i] = mesh_assets.set(bullet_mesh.0[i].clone(), new_mesh);
        }
    }
}

fn make_mesh(
    positions: &[(Vec2, f32)],
    size: Vec2, /* , total_indices: &Res<TotalIndices> */
) -> Mesh {
    let extent_x = size.x / 2.0;
    let extent_y = size.y / 2.0;

    // let uv_config = [[0., 0.0], [1., 0.0], [0., 1.0], [1., 1.0]];
    // let uvs = uv_config.repeat(bullets.len());
    //let _span = info_span!("bullet mesh chunk generation").entered();

    // let mut indices = vec![0; positions.len() * 6];
    // indices.copy_from_slice(&TOTAL_INDICES[0..positions.len() * 6]);

    //event!(Level::INFO, "bullet mesh chunk - indices done");

    let positions = positions
        .into_iter()
        .flat_map(|(position, rotation)| {
            let unit_x = Vec2::from_angle(*rotation - PI * 0.5) * extent_x;
            let unit_y = Vec2::from_angle(*rotation) * extent_y;
            let vec1 = *position - unit_x + unit_y;
            let vec2 = *position + unit_x + unit_y;
            let vec3 = *position - unit_x - unit_y;
            let vec4 = *position + unit_x - unit_y;

            [
                ([vec1.x, vec1.y, 0.0]), // [0., 0.0], [-x, +y]
                ([vec2.x, vec2.y, 0.0]), // [1., 0.0], [+x, +y]
                ([vec3.x, vec3.y, 0.0]), // [0., 1.0], [-x, -y]
                ([vec4.x, vec4.y, 0.0]), // [1., 1.0], [+x, -y]
            ]
        })
        .collect_vec();

    //event!(Level::INFO, "bullet mesh chunk - positions done");

    let indices = Indices::U32(TOTAL_INDICES.to_vec());

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    //event!(Level::INFO, "bullet mesh chunk - mesh creation done");

    mesh
}

/* fn extract_bullet_container(mut commands: Commands, bullet_container: Res<BulletContainer>) {
    commands.insert_resource(bullet_container.clone());
} */

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the value to
    // its shader-compatible equivalent. Most core math types already implement `ShaderType`.
    #[uniform(0)]
    color: Color,
    // Images can be bound as textures in shaders. If the Image's sampler is also needed, just
    // add the sampler attribute with a different binding index.
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

// All functions on `Material2d` have default impls. You only need to implement the
// functions that are relevant for your material.
impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}