use bevy::{
    camera::visibility::NoFrustumCulling,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout},
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewBindingArrayBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::ExtractedView,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

use bytemuck::{Pod, Zeroable};

use crate::cube_grid::{InstanceData, CubeGrid, CrossSectionState, compute_visible_instances};
use crate::picking::PickingState;

const SHADER_PATH: &str = "shaders/cube_grid.wgsl";

/// Uniform buffer for shader-based hover highlight.
/// Packed as 3×vec4<u32> for WGSL alignment.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct HoverUniform {
    hover_grid: [u32; 4],  // x, y, z, has_hover (0 or 1)
    grid_dims: [u32; 4],   // DIM_X, DIM_Y, DIM_Z, unused
    cube_spacing: f32,
    _pad: [f32; 3],        // align to 16 bytes
}

impl Default for HoverUniform {
    fn default() -> Self {
        Self {
            hover_grid: [0; 4],
            grid_dims: [
                crate::cube_grid::DIM_X as u32,
                crate::cube_grid::DIM_Y as u32,
                crate::cube_grid::DIM_Z as u32,
                0,
            ],
            cube_spacing: crate::cube_grid::CUBE_SPACING,
            _pad: [0.0; 3],
        }
    }
}

/// Main-world component: current hover target, extracted to render world.
#[derive(Component, Clone)]
pub(crate) struct HoverGridData(pub(crate) HoverUniform);

impl ExtractComponent for HoverGridData {
    type QueryData = &'static HoverGridData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

/// Render-world bind group for the hover uniform buffer.
#[derive(Component)]
struct HoverBindGroup {
    #[allow(dead_code)]
    buffer: Buffer,
    bind_group: BindGroup,
}

// ── Main-world components ──

#[derive(Component, Deref, Clone)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

// ── Plugin ──

pub struct CubeGridMaterialPlugin;

impl Plugin for CubeGridMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractComponentPlugin::<HoverGridData>::default(),
        ));
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCubeGrid>()
            .init_resource::<SpecializedMeshPipelines<CubeGridPipeline>>()
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(
                Render,
                (
                    queue_cube_grid.in_set(RenderSystems::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSystems::PrepareResources),
                    prepare_hover_bind_group.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

// ── Pipeline resource ──

#[derive(Resource)]
struct CubeGridPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    hover_bgl_desc: BindGroupLayoutDescriptor,
    hover_bgl: BindGroupLayout,
}

fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<MeshPipeline>,
    render_device: Res<RenderDevice>,
) {
    let entries = vec![BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }];
    let hover_bgl = render_device.create_bind_group_layout("hover_uniform", &entries);
    let hover_bgl_desc = BindGroupLayoutDescriptor {
        label: "hover_uniform".into(),
        entries,
    };

    commands.insert_resource(CubeGridPipeline {
        shader: asset_server.load(SHADER_PATH),
        mesh_pipeline: mesh_pipeline.clone(),
        hover_bgl_desc,
        hover_bgl,
    });
}

impl SpecializedMeshPipeline for CubeGridPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout.push(self.hover_bgl_desc.clone());
        Ok(descriptor)
    }
}

// ── Queue system ──

#[allow(clippy::too_many_arguments)]
fn queue_cube_grid(
    draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<CubeGridPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CubeGridPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_fn = draw_functions.read().id::<DrawCubeGrid>();

    for (view, msaa) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) =
                render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key = view_key
                | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(&pipeline_cache, &pipeline, key, &mesh.layout)
                .unwrap();

            phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_fn,
                distance: rangefinder.distance(&mesh_instance.center),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

// ── Instance buffer ──

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("cube grid instance buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

fn prepare_hover_bind_group(
    mut commands: Commands,
    query: Query<(Entity, &HoverGridData), Changed<HoverGridData>>,
    render_device: Res<RenderDevice>,
    pipeline: Res<CubeGridPipeline>,
) {
    for (entity, hover_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("hover uniform buffer"),
            contents: bytemuck::bytes_of(&hover_data.0),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(
            None::<&str>,
            &pipeline.hover_bgl,
            &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        );
        commands.entity(entity).insert(HoverBindGroup { buffer, bind_group });
    }
}

// ── Draw command ──

type DrawCubeGrid = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawMeshInstanced,
);

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = (Read<InstanceBuffer>, Read<HoverBindGroup>);

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        buffers: Option<(&'w InstanceBuffer, &'w HoverBindGroup)>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) =
            render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some((instance_buffer, hover_bind_group)) = buffers else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        // Bind hover uniform at group 3
        pass.set_bind_group(3, &hover_bind_group.bind_group, &[]);

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };
                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), *index_format);
                pass.draw_indexed(
                    index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
                    vertex_buffer_slice.range.start as i32,
                    0..instance_buffer.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}

// ── Spawn helper ──

pub fn spawn_cube_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<CubeGrid>,
    cross_section: Res<CrossSectionState>,
) {
    let visible = compute_visible_instances(&grid, &cross_section);
    info!("Spawn cube grid: {} instances", visible.len());
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        InstanceMaterialData(visible),
        HoverGridData(HoverUniform::default()),
        Transform::IDENTITY,
        NoFrustumCulling,
    ));
}

// ── Update systems (main world) ──

/// Rebuilds instance data when cross-section or grid changes (NO per-frame rebuild for hover).
pub fn update_instance_data(
    grid: Res<CubeGrid>,
    cross_section: Res<CrossSectionState>,
    mut query: Query<&mut InstanceMaterialData>,
) {
    if !cross_section.is_changed() && !grid.is_changed() {
        return;
    }
    let visible = compute_visible_instances(&grid, &cross_section);
    for mut data in &mut query {
        data.0 = visible.clone();
    }
}

/// Writes PickingState hover target into HoverGridData uniform for shader extraction.
pub fn update_hover_grid_data(
    picking: Res<PickingState>,
    mut query: Query<&mut HoverGridData>,
) {
    if !picking.is_changed() {
        return;
    }
    for mut data in &mut query {
        match picking.hovered_cube {
            Some((x, y, z)) => {
                data.0.hover_grid = [x as u32, y as u32, z as u32, 1];
            }
            None => {
                data.0.hover_grid = [0, 0, 0, 0];
            }
        }
    }
}
