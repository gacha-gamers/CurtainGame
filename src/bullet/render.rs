use std::{slice::Iter, vec::IntoIter};

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
        RenderApp,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use itertools::Itertools;

use super::{Bullet, BulletContainer};

pub struct BulletRenderPlugin;

impl Plugin for BulletRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<CustomMaterial>::default())
            .init_resource::<BulletMesh>();
            // .add_system(make_bullet_mesh);
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

fn make_bullet_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut bullet_mesh: ResMut<BulletMesh>,
    bullet_query: Query<&Bullet>,
    asset_server: Res<AssetServer>,
) {
    const CHUNK_SIZE: usize = 4096;
    // let chunk_count = (bullet_query.iter().len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let bullets_chunks = bullet_query.iter().map(|b| b.position).chunks(CHUNK_SIZE);

    for (i, chunk) in bullets_chunks.into_iter().enumerate() {
        let new_mesh = make_mesh(
            chunk.collect(),
            Vec2::new(8., 14.),
        );

        // TODO: Make this code less hideous
        // This is literally the only time if/else that I wrote in the entire codebase so far TvT
        if bullet_mesh.0.len() <= i {
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

    // bullet_mesh.0 = mesh_assets.set(bullet_mesh.0.clone(), new_mesh);
}

fn make_mesh(bullets: Vec<Vec2>, size: Vec2) -> Mesh {
    let extent_x = size.x / 2.0;
    let extent_y = size.y / 2.0;

    let uv_config = [[0., 1.0], [0., 0.0], [1., 0.0], [1., 1.0]];
    let positions = bullets
        .iter()
        .flat_map(|position| {
            [
                ([position.x - extent_x, position.y - extent_y, 0.0]),
                ([position.x - extent_x, position.y + extent_y, 0.0]),
                ([position.x + extent_x, position.y + extent_y, 0.0]),
                ([position.x + extent_x, position.y - extent_y, 0.0]),
            ]
        })
        .collect_vec();

    let indices = (0..bullets.len())
        .flat_map(|i| {
            let i = (i * 4) as u32;
            [i, i + 2, i + 1]
        })
        .collect_vec();

    let indices = Indices::U32(indices);

    let uvs = uv_config.repeat(bullets.len());

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

fn extract_bullet_container(mut commands: Commands, bullet_container: Res<BulletContainer>) {
    commands.insert_resource(bullet_container.clone());
}

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

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        println!("{:#?}", layout);
        Ok(())
    }
}
