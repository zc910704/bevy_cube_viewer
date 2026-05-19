use bevy::prelude::*;

use crate::cube_grid::{
    CubeGridDims, RangeSelectionState, SelectionMode, CUBE_SIDE, CUBE_SPACING,
};

const LINE_OFFSET: f32 = 2.0; // distance outside the grid face

/// Whether guide lines are visible. Default: on at startup.
#[derive(Resource, Clone)]
pub struct ShowGuideLine(pub bool);

impl Default for ShowGuideLine {
    fn default() -> Self {
        Self(true)
    }
}

// ── Axis position helpers ───────────────────────────────────────────

/// World-space endpoints of the **Y Axis** guide line.
/// Runs along world Z (grid Y), positioned outside the min-X / max-Z corner.
fn y_axis_line(state: &RangeSelectionState, dims: CubeGridDims) -> (Vec3, Vec3) {
    let (gx_min, _) = visible_range(state, Axis::X, dims);
    let (gy_min, gy_max) = visible_range(state, Axis::Y, dims);
    let (_, gz_max) = visible_range(state, Axis::Z, dims);

    let half_side = CUBE_SIDE / 2.0;
    let world_x = grid_to_world_x(gx_min as f32, dims) - half_side - LINE_OFFSET;
    let world_y = grid_to_world_y(gz_max as f32, dims) + half_side + LINE_OFFSET;
    let z_start = grid_to_world_z(gy_min as f32, dims) - CUBE_SPACING;
    let z_end = grid_to_world_z(gy_max as f32, dims) + CUBE_SPACING;

    (
        Vec3::new(world_x, world_y, z_start),
        Vec3::new(world_x, world_y, z_end),
    )
}

/// World-space endpoints of the **X Axis** guide line.
/// Runs along world X (grid X), positioned outside the max-Y / max-Z corner.
fn x_axis_line(state: &RangeSelectionState, dims: CubeGridDims) -> (Vec3, Vec3) {
    let (gx_min, gx_max) = visible_range(state, Axis::X, dims);
    let (_, gy_max) = visible_range(state, Axis::Y, dims);
    let (_, gz_max) = visible_range(state, Axis::Z, dims);

    let half_side = CUBE_SIDE / 2.0;
    let world_y = grid_to_world_y(gz_max as f32, dims) + half_side + LINE_OFFSET;
    let world_z = grid_to_world_z(gy_max as f32, dims) + half_side + LINE_OFFSET;
    let x_start = grid_to_world_x(gx_min as f32, dims) - CUBE_SPACING;
    let x_end = grid_to_world_x(gx_max as f32, dims) + CUBE_SPACING;

    (
        Vec3::new(x_start, world_y, world_z),
        Vec3::new(x_end, world_y, world_z),
    )
}

// ── Grid-to-world conversions ────────────────────────────────────────

#[inline]
fn grid_to_world_x(gx: f32, dims: CubeGridDims) -> f32 {
    (gx - (dims.x - 1) as f32 / 2.0) * CUBE_SPACING
}

#[inline]
fn grid_to_world_y(gz: f32, dims: CubeGridDims) -> f32 {
    (gz - (dims.z - 1) as f32 / 2.0) * CUBE_SPACING
}

#[inline]
fn grid_to_world_z(gy: f32, dims: CubeGridDims) -> f32 {
    (gy - (dims.y - 1) as f32 / 2.0) * CUBE_SPACING
}

// ── Label positions ──────────────────────────────────────────────────

/// World-space positions for the X and Y axis labels.
pub fn guide_line_label_positions(
    state: &RangeSelectionState,
    dims: CubeGridDims,
) -> (Vec3, Vec3) {
    let (ys, ye) = y_axis_line(state, dims);
    let y_mid = (ys + ye) / 2.0;
    let y_label = y_mid + Vec3::new(1.5, 0.5, 0.0);

    let (xs, xe) = x_axis_line(state, dims);
    let x_mid = (xs + xe) / 2.0;
    let x_label = x_mid + Vec3::new(0.0, 0.5, 1.5);

    (x_label, y_label)
}

// ── Systems ──────────────────────────────────────────────────────────

/// Draws X and Y axis guide lines via gizmos when enabled.
pub fn draw_guide_line(
    show: Res<ShowGuideLine>,
    state: Res<RangeSelectionState>,
    grid: Res<crate::cube_grid::CubeGrid>,
    mut gizmos: Gizmos,
) {
    if !show.0 {
        return;
    }
    let dims = grid.dims;

    let (x_start, x_end) = x_axis_line(&state, dims);
    gizmos.line(x_start, x_end, Color::srgb(1.0, 0.25, 0.25));

    let (y_start, y_end) = y_axis_line(&state, dims);
    gizmos.line(y_start, y_end, Color::srgb(0.15, 1.0, 0.5));
}

// ── Visible-range helpers ────────────────────────────────────────────

fn visible_range(state: &RangeSelectionState, axis: Axis, dims: CubeGridDims) -> (usize, usize) {
    match state.mode {
        SelectionMode::Section => {
            let dim = dim_for_axis(dims, axis);
            let slider = slider_for_axis(state, axis);
            if slider == 0 {
                (0, dim - 1)
            } else {
                let v = (slider - 1) as usize;
                (v, v)
            }
        }
        SelectionMode::Range => {
            let dim = dim_for_axis(dims, axis);
            let (min, max) = range_for_axis(state, axis);
            let lo = min.saturating_sub(1) as usize;
            let hi = max.min(dim as u32).saturating_sub(1) as usize;
            (lo, hi)
        }
    }
}

fn dim_for_axis(dims: CubeGridDims, axis: Axis) -> usize {
    match axis {
        Axis::X => dims.x,
        Axis::Y => dims.y,
        Axis::Z => dims.z,
    }
}

fn slider_for_axis(state: &RangeSelectionState, axis: Axis) -> u32 {
    match axis {
        Axis::X => state.x_slider,
        Axis::Y => state.y_slider,
        Axis::Z => state.z_slider,
    }
}

fn range_for_axis(state: &RangeSelectionState, axis: Axis) -> (u32, u32) {
    match axis {
        Axis::X => (state.x_min, state.x_max),
        Axis::Y => (state.y_min, state.y_max),
        Axis::Z => (state.z_min, state.z_max),
    }
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}
