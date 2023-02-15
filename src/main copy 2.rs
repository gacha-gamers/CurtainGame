//! A shader that renders a mesh multiple times in one draw call.

mod player;

use std::f32::consts::PI;

use bevy::{
    core_pipeline::core_3d::Transparent3d,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, NoFrustumCulling},
        RenderApp, RenderStage,
    },
};
use bytemuck::{Pod, Zeroable};
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(BulletMaterialPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .add_system(move_bullets)
        .add_system(spawn_bullets)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        meshes.add(Mesh::from(shape::Quad {
            size: Vec2::splat(0.1),
            flip: false,
        })),
        SpatialBundle::VISIBLE_IDENTITY,
        BulletDataVector::from_loop(15, |p| {
            (Vec3::ZERO, Vec2::from_angle(p * 2. * PI).extend(0.) * 0.05)
        }),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("SA_bullet.png"),
            ..Default::default()
        },
        player::Player,
    ));
    /*
    (0..15)
        .map(|i| BulletData {
            position: Vec3::ZERO,
            scale: 1.0,
            color: Color::hsla(i as f32 / 15. * 360., 0.5, 1., 1.).as_rgba_f32(),
        })
        .collect() */
    // camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn spawn_bullets(mut query: Query<&mut BulletDataVector>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::E) {
        let mut bullet_datas = query.single_mut();
        bullet_datas.add_from_loop(10000, |p| {
            (Vec3::ZERO, Vec2::from_angle(p * 2. * PI).extend(0.) * 0.01)
        });
        println!("Bullet count: {}", bullet_datas.positions.len());
    }
}

fn move_bullets(mut query: Query<&mut BulletDataVector>) {
    query.for_each_mut(|mut b| b.process_velocities());
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct BulletData {
    position: Vec3,
    scale: f32,
    color: [f32; 4],
}

#[derive(Component, Default, Clone)]
struct BulletDataVector {
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
}

impl BulletDataVector {
    fn process_velocities(&mut self) {
        for (position, velocity) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *position += *velocity;
        }
    }

    fn add_from_loop<F>(&mut self, count: u32, callback: F)
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let (positions, velocities): (Vec<Vec3>, Vec<Vec3>) = (0..count)
            .map(|i| callback(i as f32 / count as f32))
            .unzip();
        self.positions.extend(positions.iter());
        self.velocities.extend(velocities.iter());
    }

    fn from_loop<F>(count: u32, callback: F) -> Self
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let mut inst = Self::default();
        inst.add_from_loop(count, callback);
        inst
    }
}

impl ExtractComponent for BulletDataVector {
    type Query = &'static BulletDataVector;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Self {
        item.clone()
    }
}

pub struct BulletMaterialPlugin;

impl Plugin for BulletMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<BulletDataVector>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustomBullets>()
            .init_resource::<BulletCustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<BulletCustomPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom_bullets)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_custom_bullets(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<BulletCustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<BulletCustomPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<BulletDataVector>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustomBullets>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct BulletInstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &BulletDataVector)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, bullet_datas) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(
                bullet_datas
                    .positions
                    .iter()
                    .map(|p| BulletData {
                        position: *p,
                        scale: 1.,
                        color: [1., 1., 1., 1.],
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            ), // bytemuck::cast_slice(bullet_datas.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        commands.entity(entity).insert(BulletInstanceBuffer {
            buffer,
            length: bullet_datas.positions.len(),
        });
    }
}

#[derive(Resource)]
pub struct BulletCustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for BulletCustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        BulletCustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for BulletCustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<BulletData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);

        Ok(descriptor)
    }
}

type DrawCustomBullets = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawBulletMeshInstanced,
);

pub struct DrawBulletMeshInstanced;

impl EntityRenderCommand for DrawBulletMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<BulletInstanceBuffer>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = instance_buffer_query.get_inner(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
