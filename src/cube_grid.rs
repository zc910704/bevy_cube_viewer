use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use rand::Rng;

/// Grid dimensions. Customizable before spawning.
#[derive(Clone, Copy, Debug)]
pub struct CubeGridDims {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl CubeGridDims {
    pub const DEFAULT: Self = Self {
        x: 100,
        y: 100,
        z: 100,
    };

    #[inline]
    pub fn total(self) -> usize {
        self.x * self.y * self.z
    }
}

pub const CUBE_SIDE: f32 = 1.0;
pub const CUBE_GAP: f32 = 0.2;
pub const CUBE_SPACING: f32 = CUBE_SIDE + CUBE_GAP; // 1.2

/// Per-instance data uploaded to GPU vertex buffer.
/// Layout: position.xyz (vec4-padded) + color.rgba (vec4).
/// Stride = 32 bytes, aligned to 16 bytes for vec4 vertex attributes.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: [f32; 4], // xyz = world position, w = unused
    pub color: [f32; 4],    // rgba
}

/// Flat u8 array for cube colors: 0 = white, 1 = red, 2 = gray.
#[derive(Resource, Clone)]
pub struct CubeGrid {
    pub dims: CubeGridDims,
    pub data: Vec<u8>,
}

impl Default for CubeGrid {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let dims = CubeGridDims::DEFAULT;
        Self {
            dims,
            data: (0..dims.total()).map(|_| rng.gen_range(0..3)).collect(),
        }
    }
}

impl CubeGrid {
    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.dims.x + z * self.dims.x * self.dims.y
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[self.index(x, y, z)]
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        let idx = self.index(x, y, z);
        self.data[idx] = value;
    }
}

/// Filter mode for cube visibility.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    #[default]
    Section,
    Range,
}

/// Grid filter state, controlled by UI sliders and ESC key.
/// Section mode: slider 0 = show all layers on this axis; 1..=DIM = show only that layer.
/// Range mode: min..=max filter; min=0 and max=DIM means no limit on that side.
#[derive(Resource, Clone)]
pub struct RangeSelectionState {
    pub x_slider: u32,
    pub y_slider: u32,
    pub z_slider: u32,
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
    pub z_min: u32,
    pub z_max: u32,
    pub mode: SelectionMode,
    pub only_failbit: bool,
    /// Set to true when sliders change — triggers instance buffer rebuild.
    pub dirty: bool,
}

impl Default for RangeSelectionState {
    fn default() -> Self {
        let dims = CubeGridDims::DEFAULT;
        Self {
            x_slider: 0,
            y_slider: 0,
            z_slider: 0,
            x_min: 1,
            x_max: dims.x as u32,
            y_min: 1,
            y_max: dims.y as u32,
            z_min: 1,
            z_max: dims.z as u32,
            mode: SelectionMode::default(),
            only_failbit: false,
            dirty: true,
        }
    }
}

/// Compute the world-space position of a cube at grid coordinates.
/// Array is centered at origin.
/// Parameters: x = left/right, y = depth, z = up/down.
#[inline]
pub fn compute_grid_position(dims: CubeGridDims, x: usize, y: usize, z: usize) -> Vec3 {
    Vec3::new(
        (x as f32 - (dims.x - 1) as f32 / 2.0) * CUBE_SPACING, // world X: left/right
        (z as f32 - (dims.z - 1) as f32 / 2.0) * CUBE_SPACING, // world Y: up/down
        (y as f32 - (dims.y - 1) as f32 / 2.0) * CUBE_SPACING, // world Z: depth
    )
}

const WHITE_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
const RED_COLOR: [f32; 4] = [0.8, 0.2, 0.2, 1.0];
const GRAY_COLOR: [f32; 4] = [0.35, 0.35, 0.35, 1.0];

/// Build the list of InstanceData for all cubes that should currently be visible.
pub fn compute_visible_instances(
    grid: &CubeGrid,
    state: &RangeSelectionState,
) -> Vec<InstanceData> {
    let dims = grid.dims;
    let (x_range, y_range, z_range) = match state.mode {
        SelectionMode::Section => (
            axis_range(state.x_slider, dims.x),
            axis_range(state.y_slider, dims.y),
            axis_range(state.z_slider, dims.z),
        ),
        SelectionMode::Range => (
            range_axis_range(state.x_min, state.x_max, dims.x),
            range_axis_range(state.y_min, state.y_max, dims.y),
            range_axis_range(state.z_min, state.z_max, dims.z),
        ),
    };

    let count = x_range.len() * y_range.len() * z_range.len();
    let mut out = Vec::with_capacity(count);

    for &z in &z_range {
        for &y in &y_range {
            for &x in &x_range {
                let pos = compute_grid_position(dims, x, y, z);
                if state.only_failbit && grid.get(x, y, z) != 1 {
                    continue;
                }
                let color = match grid.get(x, y, z) {
                    1 => RED_COLOR,
                    2 => GRAY_COLOR,
                    _ => WHITE_COLOR,
                };
                out.push(InstanceData {
                    position: [pos.x, pos.y, pos.z, 1.0],
                    color,
                });
            }
        }
    }
    out
}

fn axis_range(slider: u32, dim: usize) -> Vec<usize> {
    if slider == 0 {
        (0..dim).collect()
    } else {
        vec![(slider - 1) as usize]
    }
}

fn range_axis_range(min: u32, max: u32, dim: usize) -> Vec<usize> {
    let lo = (min.saturating_sub(1)) as usize;
    let hi = (max.min(dim as u32).saturating_sub(1)) as usize;
    (lo..=hi).collect()
}
