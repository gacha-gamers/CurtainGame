#![allow(unused_imports)]
use std::{num::NonZeroU64, ops::Range};

use bevy::{
    asset::HandleId,
    core::Pod,
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem, SystemState,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, BatchedPhaseItem, Draw, DrawFunctions, EntityRenderCommand,
            RenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureFormatPixelInfo,
        },
        view::{ExtractedView, ViewUniform, ViewUniformOffset, ViewUniforms, VisibleEntities},
        Extract, RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, DrawSprite, Mesh2dHandle, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup, SetSpriteTextureBindGroup, SetSpriteViewBindGroup,
        SPRITE_SHADER_HANDLE,
    },
    utils::{hashbrown::HashMap, FloatOrd},
};

use super::BulletPool;

#[derive(Default)]
pub struct BulletPipelinePlugin;

impl Plugin for BulletPipelinePlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(
            BULLET_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("bullet.wgsl")),
        );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<BulletPipeline>()
                .init_resource::<SpecializedRenderPipelines<BulletPipeline>>()
                .init_resource::<ExtractedBulletPools>()
                .init_resource::<BulletMeta>()
                .add_render_command::<Transparent2d, DrawBullet>()
                .add_system_to_stage(RenderStage::Extract, extract_bullets)
                .add_system_to_stage(RenderStage::Prepare, prepare_bullets)
                .add_system_to_stage(RenderStage::Queue, queue_bullets);
        }
    }
}

#[derive(Resource)]
struct BulletPipeline {
    view_layout: BindGroupLayout,
    bullet_layout: BindGroupLayout,
    material_layout: BindGroupLayout,
}

impl FromWorld for BulletPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut system_state: SystemState<Res<RenderDevice>> = SystemState::new(world);
        let render_device = system_state.get_mut(world);

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
            label: Some("bullet_view_layout"),
        });

        let bullet_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZeroU64::new(std::mem::size_of::<Vec4>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                }/* ,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZeroU64::new(std::mem::size_of::<f32>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                }, */
            ],
            label: Some("bullet_layout"),
        });

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("bullet_material_layout"),
        });

        // A 1x1x1 'all 1.0' texture to use as a dummy texture to use in place of optional StandardMaterial textures
        // let dummy_white_gpu_image = dummy_white_gpu_image(render_device, default_sampler, render_queue);

        Self {
            view_layout,
            bullet_layout,
            material_layout,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BulletPipelineKey;

impl SpecializedRenderPipeline for BulletPipeline {
    type Key = BulletPipelineKey;

    fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("bullet_pipeline".into()),
            layout: Some(vec![
                self.view_layout.clone(),
                self.bullet_layout.clone(),
                self.material_layout.clone(),
            ]),
            vertex: VertexState {
                shader: BULLET_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: BULLET_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    // TODO: HDR support
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

pub const BULLET_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x1c80141670b9f054);

#[derive(Resource, Default)]
struct ExtractedBulletPools {
    pools: Vec<ExtractedBulletPool>,
}

struct ExtractedBulletPool {
    positions: Vec<Vec2>,
    rotations: Vec<f32>,

    handle: Handle<Image>,
}

fn extract_bullets(
    mut extracted_pools: ResMut<ExtractedBulletPools>,
    pools: Extract<Query<&BulletPool>>,
) {
    extracted_pools.pools.clear();

    pools.iter().for_each(|p| {
        extracted_pools.pools.push(ExtractedBulletPool {
            positions: p.positions.clone(),
            rotations: p.rotations.clone(),
            handle: p.handle.clone(),
        })
    });
}

#[derive(Component, Default)]
struct BulletBatch {
    handle: Handle<Image>,
    range: Range<u32>,
}

#[derive(Resource)]
pub struct BulletMeta {
    view_bind_group: Option<BindGroup>,
    bullet_states_bind_group: Option<BindGroup>,
    material_bind_groups: HashMap<Handle<Image>, BindGroup>,

    positions: BufferVec<Vec4>,
    // rotations: BufferVec<f32>,
}

impl Default for BulletMeta {
    fn default() -> Self {
        Self {
            positions: BufferVec::new(BufferUsages::STORAGE),
            // rotations: BufferVec::new(BufferUsages::STORAGE),
            material_bind_groups: Default::default(),
            view_bind_group: Default::default(),
            bullet_states_bind_group: Default::default(),
        }
    }
}

fn prepare_bullets(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut bullet_meta: ResMut<BulletMeta>,
    mut extracted_pools: ResMut<ExtractedBulletPools>,
) {
    let BulletMeta {
        positions,
        // rotations,
        ..
    }: &mut BulletMeta = bullet_meta.as_mut();

    fn spawn_bullet_batch(commands: &mut Commands, handle: &Handle<Image>, range: &Range<u32>) {
        commands.spawn(BulletBatch {
            handle: handle.clone_weak(),
            range: range.clone(),
        });
    }

    let span = info_span!("buffer_clear_and_setup").entered();

    positions.clear();
    // rotations.clear();

    extracted_pools
        .pools
        .sort_by(|a, b| a.handle.cmp(&b.handle));

    let total = extracted_pools
        .pools
        .iter()
        .map(|a| a.positions.len())
        .sum::<usize>();

    positions.reserve(total, &render_device);
    // rotations.reserve(total, &render_device);

    span.exit();

    let mut range = 0..0u32;
    let mut handle: Option<Handle<Image>> = None;

    for pool in extracted_pools.pools.iter() {
        if let Some(handle) = &handle {
            if *handle != pool.handle {
                spawn_bullet_batch(&mut commands, handle, &range);
                range.start = range.end;
            }
        }

        let _span = info_span!("buffer_move").entered();

        pool.positions.iter().zip(pool.rotations.iter()).for_each(|(p, r)| {
            positions.push(p.extend(*r).extend(0.));
        });
/*         pool.rotations.iter().for_each(|r| {
            rotations.push(*r);
        }); */
        range.end += pool.positions.len() as u32 * 6;
        handle = Some(pool.handle.clone_weak());
    }

    if !range.is_empty() {
        spawn_bullet_batch(&mut commands, &handle.unwrap(), &range);
    }

    let _span = info_span!("buffer_write").entered();

    positions.write_buffer(&render_device, &render_queue);
    // rotations.write_buffer(&render_device, &render_queue);
}

fn bind_buffer<T: Pod>(buffer: &BufferVec<T>, count: u64) -> BindingResource {
    BindingResource::Buffer(BufferBinding {
        buffer: buffer.buffer().expect("missing buffer"),
        offset: 0,
        size: Some(NonZeroU64::new(std::mem::size_of::<T>() as u64 * count).unwrap()),
    })
}

#[allow(clippy::too_many_arguments)]
fn queue_bullets(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<BulletPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<BulletPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    // msaa: Res<Msaa>,
    gpu_images: Res<RenderAssets<Image>>,
    mut bullet_meta: ResMut<BulletMeta>,
    bullet_batches: Query<(Entity, &BulletBatch)>,
    mut views: Query<&mut RenderPhase<Transparent2d>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else { return };
    if bullet_meta.positions.is_empty() {
        return;
    };

    bullet_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("bullet_view_bind_group".into()),
        layout: &pipeline.view_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
    }));

    let draw_bullet = transparent_draw_functions
        .read()
        .get_id::<DrawBullet>()
        .unwrap();

    bullet_meta.bullet_states_bind_group =
        Some(render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("bullet_states_bind_group".into()),
            layout: &pipeline.bullet_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: bind_buffer(
                        &bullet_meta.positions,
                        bullet_meta.positions.len() as u64,
                    ),
                }/* ,
                BindGroupEntry {
                    binding: 1,
                    resource: bind_buffer(
                        &bullet_meta.rotations,
                        bullet_meta.positions.len() as u64,
                    ),
                }, */
            ],
        }));

    // Iterate each view (a camera is a view)
    for mut transparent_phase in views.iter_mut() {
        // Queue all entities visible to that view
        for (entity, batch) in bullet_batches.iter() {
            if !bullet_meta.material_bind_groups.contains_key(&batch.handle) {
                // Ignore batches whose image hasn't loaded yet
                let Some(gpu_image) = gpu_images.get(&batch.handle) else { continue };

                bullet_meta.material_bind_groups.insert(
                    batch.handle.clone_weak(),
                    render_device.create_bind_group(&BindGroupDescriptor {
                        label: Some("bullet_material_bind_group".into()),
                        layout: &pipeline.material_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(&gpu_image.texture_view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Sampler(&gpu_image.sampler),
                            },
                        ],
                    }),
                );
            }

            transparent_phase.add(Transparent2d {
                entity,
                draw_function: draw_bullet,
                pipeline: pipelines.specialize(&mut pipeline_cache, &pipeline, BulletPipelineKey),
                sort_key: FloatOrd(0.),
                // This material is not batched
                batch_range: Some(batch.range.clone()),
            });
        }
    }
}

type DrawBullet = (
    SetItemPipeline,
    DrawBulletBatch,
);

struct DrawBulletBatch;
impl EntityRenderCommand for DrawBulletBatch {
    type Param = (
        SRes<BulletMeta>,
        SQuery<Read<ViewUniformOffset>>,
        SQuery<Read<BulletBatch>>,
    );
    #[inline]
    fn render<'w>(
        view: Entity,
        item: Entity,
        (bullet_meta, view_query, bullet_batch_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let batch = bullet_batch_query.get(item).unwrap();
        let view_uniform = view_query.get(view).unwrap();
        let bullet_meta = bullet_meta.into_inner();

        pass.set_bind_group(
            0,
            bullet_meta.view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );

        pass.set_bind_group(
            1,
            bullet_meta.bullet_states_bind_group.as_ref().unwrap(),
            &[],
        );

        pass.set_bind_group(
            2,
            bullet_meta.material_bind_groups.get(&batch.handle).unwrap(),
            &[],
        );

        pass.draw(batch.range.clone(), 0..1);

        RenderCommandResult::Success
        /* if let Some(gpu_mesh) = bullet_meta.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        } */
    }
}

/*
pub struct DrawBullets;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawBullets {
    type Param = (SRes<BulletMeta>, SQuery<Read<SpriteBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &P,
        (sprite_meta, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_batch = query_batch.get(item.entity()).unwrap();
        let sprite_meta = sprite_meta.into_inner();
        if sprite_batch.colored {
            pass.set_vertex_buffer(0, sprite_meta.colored_vertices.buffer().unwrap().slice(..));
        } else {
            pass.set_vertex_buffer(0, sprite_meta.vertices.buffer().unwrap().slice(..));
        }
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
        RenderCommandResult::Success
    }
}
 */

/*
fn dummy_white_gpu_image(
    render_device: Res<RenderDevice>,
    default_sampler: Res<DefaultImageSampler>,
    render_queue: Res<RenderQueue>,
) -> GpuImage {
    let image = Image::new_fill(
        Extent3d::default(),
        TextureDimension::D2,
        &[255u8; 4],
        TextureFormat::bevy_default(),
    );
    let texture = render_device.create_texture(&image.texture_descriptor);
    let sampler = match image.sampler_descriptor {
        ImageSampler::Default => (**default_sampler).clone(),
        ImageSampler::Descriptor(descriptor) => render_device.create_sampler(&descriptor),
    };
    let format_size = image.texture_descriptor.format.pixel_size();
    render_queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &image.data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(
                std::num::NonZeroU32::new(image.texture_descriptor.size.width * format_size as u32)
                    .unwrap(),
            ),
            rows_per_image: None,
        },
        image.texture_descriptor.size,
    );
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    GpuImage {
        texture,
        texture_view,
        sampler,
        texture_format: image.texture_descriptor.format,
        size: Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        ),
    }
} */
