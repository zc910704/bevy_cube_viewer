use bevy::prelude::*;

use crate::cube_grid::{
    RangeSelectionState, SelectionMode, CUBE_SIDE, CUBE_SPACING, DIM_X, DIM_Y, DIM_Z,
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
fn y_axis_line(state: &RangeSelectionState) -> (Vec3, Vec3) {
    let (gx_min, _) = visible_range(state, Axis::X);
    let (gy_min, gy_max) = visible_range(state, Axis::Y);
    let (_, gz_max) = visible_range(state, Axis::Z);

    let half_side = CUBE_SIDE / 2.0;
    let world_x = grid_to_world_x(gx_min as f32) - half_side - LINE_OFFSET;
    let world_y = grid_to_world_y(gz_max as f32) + half_side + LINE_OFFSET;
    let z_start = grid_to_world_z(gy_min as f32) - CUBE_SPACING;
    let z_end = grid_to_world_z(gy_max as f32) + CUBE_SPACING;

    (
        Vec3::new(world_x, world_y, z_start),
        Vec3::new(world_x, world_y, z_end),
    )
}

/// World-space endpoints of the **X Axis** guide line.
/// Runs along world X (grid X), positioned outside the max-Y / max-Z corner.
fn x_axis_line(state: &RangeSelectionState) -> (Vec3, Vec3) {
    let (gx_min, gx_max) = visible_range(state, Axis::X);
    let (_, gy_max) = visible_range(state, Axis::Y);
    let (_, gz_max) = visible_range(state, Axis::Z);

    let half_side = CUBE_SIDE / 2.0;
    let world_y = grid_to_world_y(gz_max as f32) + half_side + LINE_OFFSET;
    let world_z = grid_to_world_z(gy_max as f32) + half_side + LINE_OFFSET;
    let x_start = grid_to_world_x(gx_min as f32) - CUBE_SPACING;
    let x_end = grid_to_world_x(gx_max as f32) + CUBE_SPACING;

    (
        Vec3::new(x_start, world_y, world_z),
        Vec3::new(x_end, world_y, world_z),
    )
}

// ── Grid-to-world conversions ────────────────────────────────────────

#[inline]
fn grid_to_world_x(gx: f32) -> f32 {
    (gx - (DIM_X - 1) as f32 / 2.0) * CUBE_SPACING
}

#[inline]
fn grid_to_world_y(gz: f32) -> f32 {
    (gz - (DIM_Z - 1) as f32 / 2.0) * CUBE_SPACING
}

#[inline]
fn grid_to_world_z(gy: f32) -> f32 {
    (gy - (DIM_Y - 1) as f32 / 2.0) * CUBE_SPACING
}

// ── Label positions ──────────────────────────────────────────────────

/// World-space positions for the X and Y axis labels.
pub fn guide_line_label_positions(state: &RangeSelectionState) -> (Vec3, Vec3) {
    let (ys, ye) = y_axis_line(state);
    let y_mid = (ys + ye) / 2.0;
    let y_label = y_mid + Vec3::new(1.5, 0.5, 0.0);

    let (xs, xe) = x_axis_line(state);
    let x_mid = (xs + xe) / 2.0;
    let x_label = x_mid + Vec3::new(0.0, 0.5, 1.5);

    (x_label, y_label)
}

// ── Systems ──────────────────────────────────────────────────────────

/// Draws X and Y axis guide lines via gizmos when enabled.
pub fn draw_guide_line(
    show: Res<ShowGuideLine>,
    state: Res<RangeSelectionState>,
    mut gizmos: Gizmos,
) {
    if !show.0 {
        return;
    }

    let (x_start, x_end) = x_axis_line(&state);
    gizmos.line(x_start, x_end, Color::srgb(1.0, 0.25, 0.25));

    let (y_start, y_end) = y_axis_line(&state);
    gizmos.line(y_start, y_end, Color::srgb(0.15, 1.0, 0.5));
}

// ── Visible-range helpers ────────────────────────────────────────────

fn visible_range(state: &RangeSelectionState, axis: Axis) -> (usize, usize) {
    match state.mode {
        SelectionMode::Section => {
            let (slider, dim) = slider_dim(state, axis);
            if slider == 0 {
                (0, dim - 1)
            } else {
                let v = (slider - 1) as usize;
                (v, v)
            }
        }
        SelectionMode::Range => {
            let (min, max, dim) = range_bounds(state, axis);
            let lo = min.saturating_sub(1) as usize;
            let hi = max.min(dim as u32).saturating_sub(1) as usize;
            (lo, hi)
        }
    }
}

fn slider_dim(state: &RangeSelectionState, axis: Axis) -> (u32, usize) {
    match axis {
        Axis::X => (state.x_slider, DIM_X),
        Axis::Y => (state.y_slider, DIM_Y),
        Axis::Z => (state.z_slider, DIM_Z),
    }
}

fn range_bounds(state: &RangeSelectionState, axis: Axis) -> (u32, u32, usize) {
    match axis {
        Axis::X => (state.x_min, state.x_max, DIM_X),
        Axis::Y => (state.y_min, state.y_max, DIM_Y),
        Axis::Z => (state.z_min, state.z_max, DIM_Z),
    }
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}
