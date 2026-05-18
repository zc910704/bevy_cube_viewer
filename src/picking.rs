use bevy::prelude::*;

use crate::cube_grid::{
    CubeGrid, RangeSelectionState, SelectionMode, DIM_X, DIM_Y, DIM_Z, CUBE_SPACING,
};

/// Current hover target in grid coordinates.
#[derive(Resource, Default)]
pub struct PickingState {
    pub hovered_cube: Option<(usize, usize, usize)>,
}

/// Grid AABB in world space, centered at origin.
fn grid_aabb() -> (Vec3, Vec3) {
    let half_x = (DIM_X - 1) as f32 / 2.0 * CUBE_SPACING + CUBE_SPACING / 2.0;
    let half_y = (DIM_Y - 1) as f32 / 2.0 * CUBE_SPACING + CUBE_SPACING / 2.0;
    let half_z = (DIM_Z - 1) as f32 / 2.0 * CUBE_SPACING + CUBE_SPACING / 2.0;
    (
        Vec3::new(-half_x, -half_z, -half_y), // min: Y→Z mapping
        Vec3::new(half_x, half_z, half_y),     // max
    )
}

/// Ray-AABB slab test. Returns (t_min, t_max) if the ray hits the box.
fn ray_aabb_intersect(origin: Vec3, dir_inv: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<(f32, f32)> {
    let t0 = (aabb_min - origin) * dir_inv;
    let t1 = (aabb_max - origin) * dir_inv;
    let t_min = t0.min(t1);
    let t_max = t0.max(t1);
    let t_enter = t_min.x.max(t_min.y).max(t_min.z);
    let t_exit = t_max.x.min(t_max.y).min(t_max.z);
    if t_enter <= t_exit && t_exit >= 0.0 {
        Some((t_enter.max(0.0), t_exit))
    } else {
        None
    }
}

/// Convert world position to grid coordinates. Returns None if outside grid bounds.
/// Uses the coordinate convention: world(x, y, z) ← grid(x, z_idx, y_idx)
/// That is: world.x ← grid.x, world.y ← grid.z, world.z ← grid.y
fn world_to_grid(world: Vec3) -> Option<(usize, usize, usize)> {
    let gx = (world.x / CUBE_SPACING + (DIM_X - 1) as f32 / 2.0).round() as isize;
    let gz = (world.y / CUBE_SPACING + (DIM_Z - 1) as f32 / 2.0).round() as isize;
    let gy = (world.z / CUBE_SPACING + (DIM_Y - 1) as f32 / 2.0).round() as isize;
    if gx >= 0 && gx < DIM_X as isize && gy >= 0 && gy < DIM_Y as isize && gz >= 0 && gz < DIM_Z as isize {
        Some((gx as usize, gy as usize, gz as usize))
    } else {
        None
    }
}

/// DDA grid traversal. Walks the ray through the 3D grid,
/// returning the first grid cell that passes the visibility filter.
fn dda_traverse(
    origin: Vec3,
    dir: Vec3,
    state: &RangeSelectionState,
    cube_grid: &CubeGrid,
) -> Option<(usize, usize, usize)> {
    let (aabb_min, aabb_max) = grid_aabb();
    let dir_inv = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
    let (t_enter, _t_exit) = ray_aabb_intersect(origin, dir_inv, aabb_min, aabb_max)?;

    // Entry point in world space, shifted slightly inside to avoid boundary ambiguity
    let entry = origin + dir * (t_enter + 0.001);
    let mut grid = world_to_grid(entry)?;

    // Step directions (in grid space, driven by world-space direction)
    // world Z → grid Y, world Y → grid Z
    let step_x: isize = if dir.x >= 0.0 { 1 } else { -1 };
    let step_y: isize = if dir.z >= 0.0 { 1 } else { -1 }; // world Z → grid Y
    let step_z: isize = if dir.y >= 0.0 { 1 } else { -1 }; // world Y → grid Z

    // t at which we cross the next voxel boundary for each axis
    let next_boundary = |grid_idx: usize, step: isize, dim: usize, world_comp: f32, dir_comp: f32| -> f32 {
        let boundary = if step > 0 {
            (grid_idx as f32 - (dim - 1) as f32 / 2.0 + 0.5) * CUBE_SPACING
        } else {
            (grid_idx as f32 - (dim - 1) as f32 / 2.0 - 0.5) * CUBE_SPACING
        };
        (boundary - world_comp) / dir_comp
    };

    // NOTE: grid.0=x (world X), grid.1=y (world Z), grid.2=z (world Y)
    let mut t_max_x = next_boundary(grid.0, step_x, DIM_X, origin.x, dir.x);
    let mut t_max_y = next_boundary(grid.1, step_y, DIM_Y, origin.z, dir.z);
    let mut t_max_z = next_boundary(grid.2, step_z, DIM_Z, origin.y, dir.y);

    let t_delta_x = (CUBE_SPACING / dir.x).abs();
    let t_delta_y = (CUBE_SPACING / dir.z).abs();
    let t_delta_z = (CUBE_SPACING / dir.y).abs();

    // Safety limit: max diagonal steps
    let max_steps = DIM_X + DIM_Y + DIM_Z;

    for _ in 0..max_steps {
        // Check bounds
        if grid.0 >= DIM_X || grid.1 >= DIM_Y || grid.2 >= DIM_Z {
            return None;
        }

        // Visibility check against cross-section state and failbit filter
        if is_visible(grid.0, grid.1, grid.2, state, cube_grid) {
            return Some(grid);
        }

        // Advance to next cell (pick axis with smallest t_max)
        if t_max_x <= t_max_y && t_max_x <= t_max_z {
            if step_x > 0 {
                if grid.0 + 1 >= DIM_X { return None; }
            } else if grid.0 == 0 {
                return None;
            }
            grid.0 = (grid.0 as isize + step_x) as usize;
            t_max_x += t_delta_x;
        } else if t_max_y <= t_max_z {
            if step_y > 0 {
                if grid.1 + 1 >= DIM_Y { return None; }
            } else if grid.1 == 0 {
                return None;
            }
            grid.1 = (grid.1 as isize + step_y) as usize;
            t_max_y += t_delta_y;
        } else {
            if step_z > 0 {
                if grid.2 + 1 >= DIM_Z { return None; }
            } else if grid.2 == 0 {
                return None;
            }
            grid.2 = (grid.2 as isize + step_z) as usize;
            t_max_z += t_delta_z;
        }
    }

    None
}

/// Check if a grid cell passes the cross-section filter and failbit filter.
fn is_visible(x: usize, y: usize, z: usize, state: &RangeSelectionState, grid: &CubeGrid) -> bool {
    let in_range = match state.mode {
        SelectionMode::Section => {
            (state.x_slider == 0 || x == (state.x_slider - 1) as usize)
            && (state.y_slider == 0 || y == (state.y_slider - 1) as usize)
            && (state.z_slider == 0 || z == (state.z_slider - 1) as usize)
        }
        SelectionMode::Range => {
            let x_lo = state.x_min.saturating_sub(1) as usize;
            let x_hi = state.x_max.min(DIM_X as u32).saturating_sub(1) as usize;
            let y_lo = state.y_min.saturating_sub(1) as usize;
            let y_hi = state.y_max.min(DIM_Y as u32).saturating_sub(1) as usize;
            let z_lo = state.z_min.saturating_sub(1) as usize;
            let z_hi = state.z_max.min(DIM_Z as u32).saturating_sub(1) as usize;
            x >= x_lo && x <= x_hi && y >= y_lo && y <= y_hi && z >= z_lo && z <= z_hi
        }
    };
    if !in_range {
        return false;
    }
    if state.only_failbit && grid.get(x, y, z) != 1 {
        return false;
    }
    true
}

/// Runs each frame: casts a ray from the camera through the cursor,
/// traverses the grid via DDA, and updates PickingState.
pub fn picking_system(
    camera: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    cross_section: Res<RangeSelectionState>,
    cube_grid: Res<CubeGrid>,
    mut picking: ResMut<PickingState>,
) {
    let Ok(window) = windows.single() else {
        picking.hovered_cube = None;
        return;
    };
    let (camera, cam_transform) = camera.into_inner();

    let Some(cursor) = window.cursor_position() else {
        picking.hovered_cube = None;
        return;
    };

    // Build world-space ray from camera via viewport_to_world
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        picking.hovered_cube = None;
        return;
    };

    // Run DDA
    picking.hovered_cube = dda_traverse(ray.origin, *ray.direction, &cross_section, &cube_grid);
}
