use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::ui_widgets::CoreSliderDragState;
use bevy::window::Window;

use crate::cube_grid::{CubeGrid, CubeGridDims, RangeSelectionState, SelectionMode, CUBE_SPACING};
use crate::ui::CubeGridSlider;

/// 平移灵敏度：1.0 = 鼠标像素与场景移动 1:1 匹配
const PAN_SENSITIVITY: f32 = 0.4;
/// 3D 轨道旋转基础灵敏度
const ORBIT_SENSITIVITY: f32 = 0.0035;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    ThreeD,
    SectionX,
    SectionY,
    SectionZ,
}

impl ViewMode {
    pub fn section_orientation(&self) -> (Vec3, Vec3) {
        match self {
            ViewMode::ThreeD => (Vec3::NEG_Z, Vec3::Y),
            ViewMode::SectionX => (Vec3::NEG_X, Vec3::Y),
            ViewMode::SectionY => (Vec3::NEG_Z, Vec3::Y),
            ViewMode::SectionZ => (Vec3::NEG_Y, Vec3::Z),
        }
    }
}

pub fn visible_center(state: &RangeSelectionState, dims: CubeGridDims) -> Vec3 {
    let (x_lo, x_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.x_slider, dims.x),
        SelectionMode::Range => range_center_bounds(state.x_min, state.x_max, dims.x),
    };
    let (y_lo, y_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.y_slider, dims.y),
        SelectionMode::Range => range_center_bounds(state.y_min, state.y_max, dims.y),
    };
    let (z_lo, z_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.z_slider, dims.z),
        SelectionMode::Range => range_center_bounds(state.z_min, state.z_max, dims.z),
    };

    let x_mid = (x_lo + x_hi) as f32 / 2.0;
    let y_mid = (y_lo + y_hi) as f32 / 2.0;
    let z_mid = (z_lo + z_hi) as f32 / 2.0;

    Vec3::new(
        (x_mid - (dims.x - 1) as f32 / 2.0) * CUBE_SPACING,
        (z_mid - (dims.z - 1) as f32 / 2.0) * CUBE_SPACING,
        (y_mid - (dims.y - 1) as f32 / 2.0) * CUBE_SPACING,
    )
}

fn axis_center_bounds(slider: u32, dim: usize) -> (usize, usize) {
    if slider == 0 {
        (0, dim - 1)
    } else {
        let v = (slider - 1) as usize;
        (v, v)
    }
}

fn range_center_bounds(min: u32, max: u32, dim: usize) -> (usize, usize) {
    let lo = min.saturating_sub(1) as usize;
    let hi = max.min(dim as u32).saturating_sub(1) as usize;
    (lo, hi)
}

#[derive(Resource)]
pub struct CameraState {
    pub orbit_distance: f32,
    pub section_distance: f32,
    pub section_target: Vec3,
    pub orbit_only_visible: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            orbit_distance: 1300.0,
            section_distance: 200.0,
            section_target: Vec3::ZERO,
            orbit_only_visible: false,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn orbit_camera(
    camera: Single<(&mut Transform, &Projection), With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
    view_mode: Res<ViewMode>,
    cross_section: Res<RangeSelectionState>,
    grid: Res<CubeGrid>,
    mut prev_mode: Local<Option<ViewMode>>,
    slider_drag: Query<&CoreSliderDragState, With<CubeGridSlider>>,
    windows: Query<&Window>,
) {
    let (mut transform, projection) = camera.into_inner();
    let delta = mouse_motion.delta;
    let dragging_slider = slider_drag.iter().any(|d| d.dragging);
    let mode = *view_mode;

    // Mode transition handling
    let mode_changed = *prev_mode != Some(mode);
    if mode_changed {
        *prev_mode = Some(mode);
        if mode != ViewMode::ThreeD {
            let fov = match projection {
                Projection::Perspective(p) => p.fov,
                _ => std::f32::consts::FRAC_PI_3,
            };
            let visible_extent = match mode {
                ViewMode::SectionX => {
                    (grid.dims.y as f32 * CUBE_SPACING).max(grid.dims.z as f32 * CUBE_SPACING)
                }
                ViewMode::SectionY => {
                    (grid.dims.x as f32 * CUBE_SPACING).max(grid.dims.z as f32 * CUBE_SPACING)
                }
                ViewMode::SectionZ => {
                    (grid.dims.x as f32 * CUBE_SPACING).max(grid.dims.y as f32 * CUBE_SPACING)
                }
                _ => 100.0,
            };
            camera_state.section_distance = (visible_extent / 2.0) / (fov / 2.0).tan();
            camera_state.section_target = Vec3::ZERO;
        } else {
            camera_state.orbit_distance = transform.translation.distance(Vec3::ZERO);
        }
    }

    match mode {
        ViewMode::ThreeD => {
            if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
                let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                let distance_scale = (1300.0 / camera_state.orbit_distance).sqrt();
                let new_yaw = yaw - delta.x * ORBIT_SENSITIVITY * distance_scale;
                let new_pitch =
                    (pitch - delta.y * ORBIT_SENSITIVITY * distance_scale).clamp(-1.5, 1.5);
                transform.rotation =
                    Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
            }

            for event in mouse_wheel_reader.read() {
                camera_state.orbit_distance =
                    (camera_state.orbit_distance - event.y * 5.0).clamp(10.0, 2000.0);
            }

            let target = if camera_state.orbit_only_visible {
                visible_center(&cross_section, grid.dims)
            } else {
                Vec3::ZERO
            };
            transform.translation = target - transform.forward() * camera_state.orbit_distance;
        }
        ViewMode::SectionX | ViewMode::SectionY | ViewMode::SectionZ => {
            let (forward, up) = mode.section_orientation();
            transform.look_at(camera_state.section_target + forward, up);

            if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
                let right = transform.rotation * Vec3::X;
                let cam_up = transform.rotation * Vec3::Y;
                // Pan speed based on visible world extent at focal plane:
                // 1 pixel drag = 1 pixel world movement at the target distance
                let fov = match projection {
                    Projection::Perspective(p) => p.fov,
                    _ => std::f32::consts::FRAC_PI_3,
                };
                let viewport_h = windows.single().map(|w| w.height()).unwrap_or(1080.0);
                let pan_speed = 2.0 * camera_state.section_distance
                    * (fov / 2.0).tan() / viewport_h
                    * PAN_SENSITIVITY;
                camera_state.section_target -=
                    right * delta.x * pan_speed - cam_up * delta.y * pan_speed;
            }

            for event in mouse_wheel_reader.read() {
                camera_state.section_distance =
                    (camera_state.section_distance - event.y * 5.0).clamp(10.0, 2000.0);
            }

            transform.translation =
                camera_state.section_target - forward * camera_state.section_distance;
        }
    }
}
