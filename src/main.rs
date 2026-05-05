mod shared_state;
mod cube_renderer;

use bevy::{
    prelude::*,
    input::mouse::AccumulatedMouseMotion,
    input::mouse::MouseWheel,
};
use bevy_diagnostic::LogDiagnosticsPlugin;

use shared_state::SharedState;
use cube_renderer::CubeRendererPlugin;

#[derive(Resource)]
struct CameraState {
    orbit_distance: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self { orbit_distance: 20.0 }
    }
}

fn main() {
    let shared_state = SharedState::default();

    App::new()
        .insert_resource(shared_state)
        .init_resource::<CameraState>()
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            CubeRendererPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}

fn orbit_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
) {
    let delta = mouse_motion.delta;

    // 鼠标左键旋转
    if mouse_buttons.pressed(MouseButton::Left) {
        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        let new_yaw = yaw - delta.x * 0.004;
        let new_pitch = (pitch - delta.y * 0.003).clamp(-1.5, 1.5);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
    }

    // 滚轮调整距离
    for event in mouse_wheel_reader.read() {
        camera_state.orbit_distance = (camera_state.orbit_distance - event.y * 2.0).clamp(5.0, 100.0);
    }

    // 更新相机位置
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_state.orbit_distance;
}