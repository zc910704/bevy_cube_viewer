use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::ui_widgets::CoreSliderDragState;

use crate::ui::CubeGridSlider;

#[derive(Resource)]
pub struct CameraState {
    pub orbit_distance: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self { orbit_distance: 80.0 }
    }
}

pub fn orbit_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
    slider_drag: Query<&CoreSliderDragState, With<CubeGridSlider>>,
) {
    let delta = mouse_motion.delta;
    let dragging_slider = slider_drag.iter().any(|d| d.dragging);

    if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        let new_yaw = yaw - delta.x * 0.004;
        let new_pitch = (pitch - delta.y * 0.003).clamp(-1.5, 1.5);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
    }

    for event in mouse_wheel_reader.read() {
        camera_state.orbit_distance =
            (camera_state.orbit_distance - event.y * 5.0).clamp(10.0, 2000.0);
    }

    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_state.orbit_distance;
}
