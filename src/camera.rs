use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::ui_widgets::CoreSliderDragState;

use crate::cube_grid::{DIM_X, DIM_Y, DIM_Z, CUBE_SPACING};
use crate::ui::CubeGridSlider;

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

#[derive(Resource)]
pub struct CameraState {
    pub orbit_distance: f32,
    pub section_distance: f32,
    pub section_target: Vec3,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            orbit_distance: 80.0,
            section_distance: 200.0,
            section_target: Vec3::ZERO,
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
    mut prev_mode: Local<Option<ViewMode>>,
    slider_drag: Query<&CoreSliderDragState, With<CubeGridSlider>>,
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
                    (DIM_Y as f32 * CUBE_SPACING).max(DIM_Z as f32 * CUBE_SPACING)
                }
                ViewMode::SectionY => {
                    (DIM_X as f32 * CUBE_SPACING).max(DIM_Z as f32 * CUBE_SPACING)
                }
                ViewMode::SectionZ => {
                    (DIM_X as f32 * CUBE_SPACING).max(DIM_Y as f32 * CUBE_SPACING)
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
                let new_yaw = yaw - delta.x * 0.004;
                let new_pitch = (pitch - delta.y * 0.003).clamp(-1.5, 1.5);
                transform.rotation =
                    Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
            }

            for event in mouse_wheel_reader.read() {
                camera_state.orbit_distance =
                    (camera_state.orbit_distance - event.y * 5.0).clamp(10.0, 2000.0);
            }

            let target = Vec3::ZERO;
            transform.translation = target - transform.forward() * camera_state.orbit_distance;
        }
        ViewMode::SectionX | ViewMode::SectionY | ViewMode::SectionZ => {
            let (forward, up) = mode.section_orientation();
            transform.look_at(camera_state.section_target + forward, up);

            if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
                let right = transform.rotation * Vec3::X;
                let cam_up = transform.rotation * Vec3::Y;
                let speed = camera_state.section_distance * 0.002;
                camera_state.section_target -=
                    right * delta.x * speed - cam_up * delta.y * speed;
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
